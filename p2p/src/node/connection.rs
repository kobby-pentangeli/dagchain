use super::{event::Event, message::Message};
use crate::error::P2pError;
use bytes::Bytes;
use crossbeam_channel::{self, Sender};
use crypto::hash::Hash;
use quic_p2p::{Peer, QuicP2p, QuicP2pError as QuicError};
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::Entry, HashMap};
use std::net::SocketAddr;

pub(super) const MAX_CONNECTION_LEN: usize = 5;

pub type ConnectionMap = HashMap<SocketAddr, (Option<Hash>, ConnectionState)>;

/// Manages the connections of a node
pub(super) struct Connection {
    entries: ConnectionMap,
    active_connections: HashMap<Hash, SocketAddr>,
    routing_table: RoutingTable,
}

impl Connection {
    pub fn new() -> Self {
        Self {
            entries: Default::default(),
            active_connections: Default::default(),
            routing_table: Default::default(),
        }
    }

    pub fn our_routing_table(&self) -> RoutingTable {
        self.routing_table.clone()
    }

    pub fn routing_table(&self) -> &RoutingTable {
        &self.routing_table
    }

    pub fn our_connections(&self) -> &ConnectionMap {
        &self.entries
    }

    pub fn update_routing_table(
        &mut self,
        peer_routing_table: SharedRoutingTable,
        peer_id: Hash,
        quic: &mut QuicP2p,
        our_id: &Hash,
    ) {
        let _ = peer_routing_table
            .entries()
            .keys()
            .into_iter()
            .map(|entry| {
                if !self.routing_table.has_node(entry) {
                    self.routing_table.add_new_node(entry);
                }
            })
            .collect::<Vec<_>>();
        let mut changed = false;
        let _ = self
            .routing_table
            .entries_mut()
            .iter_mut()
            .map(|(dest, (hop_to, hop_count))| {
                if let Some(new_hop_count) = peer_routing_table.get_routing_info(dest) {
                    if new_hop_count + 1 < *hop_count {
                        changed = true;
                        let _ = std::mem::replace(hop_to, peer_id);
                        let _ = std::mem::replace(hop_count, new_hop_count + 1);
                    }
                }
            })
            .collect::<Vec<_>>();
        if changed {
            self.routing_table.increment_version();
            self.share_routing_table(quic, our_id);
        }
    }

    pub fn get_active_connections(&self) -> &HashMap<Hash, SocketAddr> {
        &self.active_connections
    }

    pub fn bootstrap(&mut self, contacts: Vec<SocketAddr>, quic: &mut QuicP2p) {
        for node in contacts {
            if self.entries.len() == MAX_CONNECTION_LEN {
                break;
            }
            if !self.entries.contains_key(&node) {
                self.bootstrap_with(node, quic);
            }
        }
    }

    pub fn bootstrap_with(&mut self, socket_addr: SocketAddr, quic: &mut QuicP2p) {
        let _ = self
            .entries
            .insert(socket_addr, (None, ConnectionState::Connecting));
        quic.connect_to(socket_addr);
    }

    pub fn connect_to(&mut self, conn_info: &ConnectionInfo, quic: &mut QuicP2p) {
        log::trace!("Connecting to: {:?}", conn_info);
        let _ = self.entries.insert(
            conn_info.socket_addr,
            (Some(conn_info.hash), ConnectionState::Connecting),
        );
        quic.connect_to(conn_info.socket_addr);
    }

    pub fn handle_successful_connection(
        &mut self,
        peer: &Peer,
        our_id: &Hash,
        node_tx: &Sender<Event>,
        quic: &mut QuicP2p,
    ) -> Result<(), P2pError> {
        let socket_addr = peer.peer_addr();
        let connection_entry = self.entries.get_mut(&socket_addr);
        let mut connected = false;
        if let Some((public_key, state)) = connection_entry {
            quic.send(
                Peer::Node(socket_addr),
                Bytes::from(
                    bincode::serialize(&Message::Identification(*our_id))
                        .map_err(|e| P2pError::BincodeError(e))?,
                ),
                0,
            );
            if let Some(key) = public_key {
                let _ = std::mem::replace(state, ConnectionState::Connected);
                let _ = self.active_connections.insert(*key, socket_addr);
                node_tx
                    .send(Event::ConnectedTo(*key))
                    .map_err(|e| P2pError::CrossbeamSenderError(e))?;
                self.routing_table.add_direct_connection(key);
                self.routing_table.increment_version();
                connected = true;
                log::debug!("Successfully connected with peer {:?}", socket_addr);
                log::debug!("Our connections: {:?}", &self.entries);
            } else {
                log::debug!("Waiting for identification from peer: {:?}", &socket_addr);
            }
        } else {
            if self.entries.len() == MAX_CONNECTION_LEN {
                let our_connections = self.entries.keys().cloned().collect::<Vec<_>>();
                log::warn!(
                    "Too many connections. Disconnecting from {:?}",
                    &socket_addr
                );
                quic.send(
                    Peer::Node(socket_addr),
                    Bytes::from(
                        bincode::serialize(&Message::Contacts(our_connections))
                            .map_err(|e| P2pError::BincodeError(e))?,
                    ),
                    1,
                );
                return Ok(());
            }
            let _ = self
                .entries
                .insert(socket_addr, (None, ConnectionState::Incoming));
            quic.send(
                Peer::Node(socket_addr),
                Bytes::from(
                    bincode::serialize(&Message::Identification(*our_id))
                        .map_err(|e| P2pError::BincodeError(e))?,
                ),
                0,
            );
        }
        if connected {
            self.share_routing_table(quic, our_id);
        }
        Ok(())
    }

    pub fn handle_peer_identification(
        &mut self,
        our_hash: Hash,
        peer: &Peer,
        peer_hash: Hash,
        node_tx: &Sender<Event>,
        quic: &mut QuicP2p,
    ) -> Result<(), P2pError> {
        log::debug!(
            "Peer {:?} has identified itself as {:?}",
            peer.peer_addr(),
            &peer_hash
        );
        let mut connected = false;
        if let Entry::Occupied(mut entry) = self.entries.entry(peer.peer_addr()) {
            let (key, state) = entry.get_mut();
            if key.is_none() {
                let _ = std::mem::replace(key, Some(peer_hash));
                let _ = std::mem::replace(state, ConnectionState::Connected);
                node_tx
                    .send(Event::ConnectedTo(peer_hash))
                    .map_err(|e| P2pError::CrossbeamSenderError(e))?;
                let _ = self.active_connections.insert(peer_hash, peer.peer_addr());
                self.routing_table.add_direct_connection(&peer_hash);
                self.routing_table.increment_version();
                connected = true;
                log::debug!("Successfully connected with peer {:?}", peer.peer_addr());
                log::debug!("Our connections: {:?}", &self.entries);
            }
        }
        if connected {
            self.share_routing_table(quic, &our_hash);
        }
        Ok(())
    }

    pub fn share_routing_table(&mut self, quic: &mut QuicP2p, our_id: &Hash) {
        let routing_table = self.routing_table.clone();
        for socket in self.get_active_connections().values() {
            quic.send(
                Peer::Node(*socket),
                Bytes::from(
                    bincode::serialize(&Message::RoutingTable {
                        routing_table: routing_table.get_shared(),
                        source: *our_id,
                    })
                    .unwrap(),
                ),
                0,
            );
        }
    }

    pub fn handle_connection_failure(
        &mut self,
        peer: Peer,
        error: QuicError,
    ) -> Result<(), P2pError> {
        let peer_addr = peer.peer_addr();
        log::info!(
            "Lost connection with Peer at {:?} due to {:?}",
            &peer_addr,
            &error
        );
        if let Some((id, _)) = self.entries.remove(&peer_addr) {
            log::info!("Disconnected from peer: {:?}", id);
        } else {
            log::warn!(
                "We did not maintain the connection with peer at {:?}",
                &peer_addr
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoutingTable {
    entries: HashMap<Hash, (Hash, usize)>,
    version: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SharedRoutingTable {
    entries: HashMap<Hash, usize>,
}

impl SharedRoutingTable {
    fn entries(&self) -> &HashMap<Hash, usize> {
        &self.entries
    }

    fn get_routing_info(&self, node_id: &Hash) -> Option<usize> {
        self.entries.get(node_id).map(|hops| *hops)
    }
}

impl RoutingTable {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            version: 0,
        }
    }

    pub fn get_shared(&self) -> SharedRoutingTable {
        let entries = self
            .entries
            .iter()
            .map(|(node_id, (_intermediate, hops))| (*node_id, *hops))
            .collect::<HashMap<Hash, usize>>();
        SharedRoutingTable { entries }
    }

    pub fn get_routing_info(&self, node_id: &Hash) -> Option<&(Hash, usize)> {
        self.entries.get(node_id)
    }

    pub fn entries_mut(&mut self) -> &mut HashMap<Hash, (Hash, usize)> {
        &mut self.entries
    }

    pub fn entries(&self) -> &HashMap<Hash, (Hash, usize)> {
        &self.entries
    }

    pub fn has_node(&self, node_id: &Hash) -> bool {
        self.entries.contains_key(node_id)
    }

    pub fn add_new_node(&mut self, node_id: &Hash) {
        let _ = self
            .entries
            .insert(*node_id, (Hash::generate_random(), usize::MAX));
    }

    pub fn add_direct_connection(&mut self, node_id: &Hash) {
        let _ = self.entries.insert(*node_id, (*node_id, 1));
    }

    pub fn increment_version(&mut self) {
        self.version += 1;
    }

    pub fn version(&self) -> usize {
        self.version
    }
}

impl Default for RoutingTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ConnectionInfo {
    pub hash: Hash,
    pub socket_addr: SocketAddr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConnectionState {
    Connecting,
    Incoming,
    Connected,
}
