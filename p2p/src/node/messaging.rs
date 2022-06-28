use super::{connection::RoutingTable, event::Event, identity::Identity, message::Message};
use crate::error::P2pError;
use bytes::Bytes;
use crossbeam_channel::Sender;
use crypto::{hash::Hash, signature::Signature};
use quic_p2p::{Peer, QuicP2p};
use std::collections::{hash_map::Entry, HashMap};
use std::net::SocketAddr;

const TTL: usize = 5;

pub(super) struct Messaging {
    outbox: HashMap<Hash, Vec<(Hash, Message, usize)>>,
    pending_messages: Vec<(Bytes, u64, SocketAddr)>,
}

impl Messaging {
    pub fn new() -> Self {
        Self {
            outbox: Default::default(),
            pending_messages: Default::default(),
        }
    }

    pub fn handle_unsent_message(
        &mut self,
        msg: Bytes,
        token: u64,
        addr: SocketAddr,
    ) -> Result<(), P2pError> {
        self.pending_messages.push((msg, token, addr));
        Ok(())
    }

    pub fn handle_agent_message(
        &mut self,
        our_id: &Identity,
        peer: &Peer,
        mut payload: Vec<(Hash, Message, usize)>,
        active_connections: &HashMap<Hash, SocketAddr>,
        quic: &mut QuicP2p,
        node_tx: &Sender<Event>,
        routing_table: RoutingTable,
    ) {
        let our_hash = our_id.get_our_hash().unwrap();
        while let Some((target, message, step)) = payload.pop() {
            if target == our_hash {
                self.handle_message(peer, message, our_id, node_tx)
                    .unwrap_or_else(|err| {
                        log::error!("Error: {:?}", err);
                    });
            } else {
                if step >= 1 {
                    let (next_hop, _) = routing_table.get_routing_info(&target).unwrap();
                    match self.outbox.entry(*next_hop) {
                        Entry::Occupied(mut entry) => {
                            let messages = entry.get_mut();
                            messages.push((target, message, step - 1));
                        }
                        Entry::Vacant(entry) => {
                            let _ = entry.insert(vec![(target, message, step - 1)]);
                        }
                    }
                }
            }
            let outbox = std::mem::replace(&mut self.outbox, HashMap::new());
            for (target, payload) in outbox {
                self.send_agent_message(active_connections, &target, quic, payload);
            }
        }
    }

    fn handle_message(
        &mut self,
        peer: &Peer,
        msg: Message,
        _our_id: &Identity,
        node_tx: &Sender<Event>,
    ) -> Result<(), P2pError> {
        match msg {
            Message::UserMessage(content) => {
                log::trace!("Peer {:?} sent us: {:?}", peer.peer_addr(), &content[..4]);
                node_tx
                    .send(Event::NewMessage(content))
                    .map_err(|e| P2pError::CrossbeamSenderError(e))?;
                Ok(())
            }
            Message::SignedMessage {
                message,
                signature,
                sender,
            } => {
                log::trace!(
                    "Peer {:?} sent us a signed message: {:?}",
                    peer.peer_addr(),
                    &message[..4]
                );
                let signature = Signature::from_bytes(&signature)
                    .map_err(|e| P2pError::CustomError(e.to_string()))?;
                if signature.verify(&sender.public_key, &message) {
                    node_tx
                        .send(Event::NewMessage(message))
                        .map_err(|e| P2pError::CrossbeamSenderError(e))?;
                } else {
                    log::error!("Message has invalid signature! Dropped.")
                }
                Ok(())
            }
            Message::ConsensusRequest { data } => {
                node_tx
                    .send(Event::ConsensusRequest(data))
                    .map_err(|e| P2pError::CrossbeamSenderError(e))?;
                Ok(())
            }
            Message::DagConsensusRequest {
                data,
                tx,
                sender,
                count,
            } => {
                let event = Event::DagConsensusRequest {
                    data,
                    tx,
                    sender,
                    count,
                };
                log::error!("Received: {:?}", event);
                node_tx
                    .send(event)
                    .map_err(|e| P2pError::CrossbeamSenderError(e))?;
                Ok(())
            }
            Message::DagConsensusResponse {
                hash,
                sender,
                strongly_preferred,
            } => {
                let event = Event::DagConsensusResponse {
                    hash,
                    sender,
                    accepted: strongly_preferred,
                };
                log::error!("Received: {:?}", event);
                node_tx
                    .send(event)
                    .map_err(|e| P2pError::CrossbeamSenderError(e))?;
                Ok(())
            }
            Message::InitBenchmarking(count, interval) => {
                node_tx
                    .send(Event::InitBenchmarkingSignal(count, interval))
                    .map_err(|e| P2pError::CrossbeamSenderError(e))?;
                Ok(())
            }
            Message::CompleteRound => {
                node_tx
                    .send(Event::CompleteRound)
                    .map_err(|e| P2pError::CrossbeamSenderError(e))?;
                Ok(())
            }
            Message::BenchmarkStats(txns) => {
                node_tx
                    .send(Event::BenchmarkStats(txns))
                    .map_err(|e| P2pError::CrossbeamSenderError(e))?;
                Ok(())
            }
            Message::BatchedConsensusRequest {
                sender,
                data,
                count,
            } => {
                node_tx
                    .send(Event::BatchedConsensusRequest {
                        sender,
                        data,
                        count,
                    })
                    .map_err(|e| P2pError::CrossbeamSenderError(e))?;
                Ok(())
            }
            Message::BatchedConsensusResponse { sender, data } => {
                node_tx
                    .send(Event::BatchedConsensusResponse { sender, data })
                    .map_err(|e| P2pError::CrossbeamSenderError(e))?;
                Ok(())
            }
            _ => {
                log::error!("Unexpected message!!");
                Ok(())
            }
        }
    }

    pub fn send_message(&mut self, dst_peer: &Hash, msg: &[u8], routing_table: &RoutingTable) {
        let (next_hop, _) = routing_table.get_routing_info(&dst_peer).unwrap();
        match self.outbox.entry(*next_hop) {
            Entry::Occupied(mut entry) => {
                let messages = entry.get_mut();
                messages.push((*dst_peer, Message::UserMessage(msg.to_vec()), TTL));
            }
            Entry::Vacant(entry) => {
                let _ = entry.insert(vec![(*dst_peer, Message::UserMessage(msg.to_vec()), TTL)]);
            }
        }
    }

    pub fn push_to_outbox(
        &mut self,
        dst_peer: Hash,
        message: Message,
        routing_table: &RoutingTable,
        active_connections: &HashMap<Hash, SocketAddr>,
        quic: &mut QuicP2p,
    ) {
        log::error!("Pushed {:?} to outbox for {:?}", message, dst_peer);
        let (next_hop, _) = routing_table.get_routing_info(&dst_peer).unwrap();
        match self.outbox.entry(*next_hop) {
            Entry::Occupied(mut entry) => {
                let messages = entry.get_mut();
                messages.push((dst_peer, message, TTL));
            }
            Entry::Vacant(entry) => {
                let _ = entry.insert(vec![(dst_peer, message, TTL)]);
            }
        }
        let payload = self.outbox.remove(next_hop).unwrap();
        self.send_agent_message(active_connections, next_hop, quic, payload);
    }

    fn send_pending_messages(&mut self, quic: &mut QuicP2p) {
        if self.pending_messages.is_empty() {
            return;
        }
        while let Some((msg, token, addr)) = self.pending_messages.pop() {
            quic.send(Peer::Node(addr), msg, token);
        }
    }

    pub fn send_agent_message(
        &mut self,
        active_connections: &HashMap<Hash, SocketAddr>,
        target: &Hash,
        quic: &mut QuicP2p,
        payload: Vec<(Hash, Message, usize)>,
    ) {
        self.send_pending_messages(quic);
        let socket = active_connections.get(target).unwrap();
        quic.send(
            Peer::Node(*socket),
            Bytes::from(bincode::serialize(&Message::AgentMessage { payload }).unwrap()),
            0,
        );
    }
}
