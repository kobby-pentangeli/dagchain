#![warn(clippy::all)]

pub mod account;
pub mod clock;
pub mod config;
pub mod network;
pub mod transaction;
pub mod tree;

use account::AccountStateChoice;
use config::ConsensusConfig;
use crypto::hash::Hash;
use network::{CommonConsensusNetwork, ConsensusNetwork};
use std::collections::{HashMap, HashSet};
use transaction::Transaction;
use tree::HashTreeNode;

pub type AccountConflictSet = HashMap<Hash, HashSet<Hash>>;

pub trait Consensus {
    fn new(config: ConsensusConfig) -> Self
    where
        Self: Sized;

    fn query(&mut self, state: &AccountStateChoice) -> &mut Self
    where
        Self: Sized;

    fn send_consensus_requests<T, N>(
        &mut self,
        state: &AccountStateChoice,
        tx: &Transaction,
        network: &mut T,
        common_network: &mut N,
        count: usize,
    ) where
        T: ConsensusNetwork,
        N: CommonConsensusNetwork;

    fn complete_dag_consensus(
        &self,
        preferred: usize,
        state: &AccountStateChoice,
        tree: &mut HashTreeNode,
    ) -> ConsensusStatus;

    fn fire_consensus<T, N>(
        &mut self,
        state: &AccountStateChoice,
        network: &mut T,
        common_network: &mut N,
        tree: Option<&mut HashTreeNode>,
    ) -> ConsensusStatus
    where
        T: ConsensusNetwork,
        N: CommonConsensusNetwork;

    fn on_query(&mut self, state: &AccountStateChoice) -> (Hash, bool);

    fn target_count(&self) -> usize;
}

#[derive(Debug)]
pub enum ConsensusStatus {
    InProgress,
    Accept(Hash),
    Reject,
}
