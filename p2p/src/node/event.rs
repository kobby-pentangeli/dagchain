use consensus::{account::AccountStateChoice, transaction::Transaction};
use crypto::hash::Hash;
use std::collections::HashSet;

/// P2p Events
#[derive(Debug, PartialEq)]
pub enum Event {
    ConnectedTo(Hash),
    NewMessage(Vec<u8>),
    ConsensusRequest(AccountStateChoice),
    DagConsensusRequest {
        sender: Hash,
        data: AccountStateChoice,
        tx: Transaction,
        count: usize,
    },
    DagConsensusResponse {
        hash: Hash,
        sender: Hash,
        accepted: bool,
    },
    TransactionComplete(Hash),
    InitBenchmarkingSignal(usize, u64),
    CompleteRound,
    BenchmarkStats(HashSet<u64>),
    BatchedConsensusReqest {
        sender: Hash,
        data: Vec<(AccountStateChoice, Transaction)>,
        count: usize,
    },
    BatchedConsensusResponse {
        sender: Hash,
        data: Vec<(Hash, bool)>,
    },
}
