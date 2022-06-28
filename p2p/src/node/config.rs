use quic_p2p::Config as QuicConfig;
use std::collections::hash_set::{self, HashSet};
use std::iter::IntoIterator;
use std::net::SocketAddr;
use structopt::StructOpt;

/// P2p node configuration
#[derive(Clone, Debug, Default, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct P2pConfig {
    #[structopt(short, long, default_value = "[]", parse(try_from_str = serde_json::from_str))]
    bootstrap_nodes: HashSet<SocketAddr>,
    #[structopt(parse(try_from_str = serde_json::from_str))]
    quic: QuicConfig,
    #[structopt(short, long)]
    deploy_agent: bool,
}

impl P2pConfig {
    pub fn get_quic_config(&self) -> QuicConfig {
        self.quic.clone()
    }

    pub fn set_quic_config(&mut self, qconfig: QuicConfig) {
        self.quic = qconfig;
    }

    pub fn get_bootstrap_contacts(&self) -> hash_set::Iter<SocketAddr> {
        self.bootstrap_nodes.iter()
    }

    pub fn add_bootstrap_contacts(&mut self, peers: impl IntoIterator<Item = SocketAddr>) {
        let _ = self.bootstrap_nodes.extend(peers);
    }

    pub fn bootstrap_nodes_mut(&mut self) -> &mut HashSet<SocketAddr> {
        &mut self.bootstrap_nodes
    }

    pub fn set_deploy_agent(&mut self) {
        self.deploy_agent = true;
    }

    pub fn should_deploy(&self) -> bool {
        self.deploy_agent
    }
}
