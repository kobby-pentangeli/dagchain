#![warn(clippy::all)]

pub mod account;
pub mod clock;
pub mod config;
pub mod dag_consensus;
pub mod network;
pub mod quantum;
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

    fn query(&self, state: &AccountStateChoice) -> &Self
    where
        Self: Sized;

    fn send_consensus_requests<T, N>(
        &self,
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
        acceptance: usize,
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

    fn on_query(&self, state: &AccountStateChoice) -> (Hash, bool);

    fn target_count(&self) -> usize;
}

#[derive(Debug)]
pub enum ConsensusStatus {
    InProgress,
    Accept(Hash),
    Reject,
}
