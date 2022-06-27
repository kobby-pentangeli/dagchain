use super::{connection::SharedRoutingTable, identity::PublicId};
use consensus::{account::AccountStateChoice, transaction::Transaction};
use crypto::hash::Hash;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::SocketAddr;

#[derive(Clone, Deserialize, Serialize)]
pub enum Message {
    UserMessage(Vec<u8>),
    EncryptedMessage(Vec<u8>),
    AuthenticatedMessage {
        message: Vec<u8>,
        sender: PublicId,
    },
    SignedMessage {
        message: Vec<u8>,
        signature: Vec<u8>,
        sender: PublicId,
    },
    Identification(Hash),
    Contacts(Vec<SocketAddr>),
    AgentMessage {
        payload: Vec<(Hash, Message, usize)>,
    },
    RoutingTable {
        routing_table: SharedRoutingTable,
        source: Hash,
    },
    ConsensusRequest {
        data: AccountStateChoice,
    },
    DagConsensusRequest {
        sender: Hash,
        data: AccountStateChoice,
        tx: Transaction,
        count: usize,
    },
    DagConsensusResponse {
        sender: Hash,
        hash: Hash,
        strongly_preferred: bool,
    },
    InitBenchmarking(usize, u64),
    CompleteRound,
    BenchmarkStats(HashSet<u64>),
    BatchedConsensusRequest {
        sender: Hash,
        data: Vec<(AccountStateChoice, Transaction)>,
        count: usize,
    },
    BatchedConsensusResponse {
        sender: Hash,
        data: Vec<(Hash, bool)>,
    },
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Message::*;
        match self {
            UserMessage(_) => write!(f, "UserMessage(..)",),
            EncryptedMessage(_) => write!(f, "EncryptedMessage(..)",),
            Identification(_) => write!(f, "Identification(..)",),
            Contacts(_) => write!(f, "Contacts(..)",),
            AuthenticatedMessage { .. } => write!(f, "AuthenticatedMessage {{ .. }} "),
            SignedMessage { .. } => write!(f, "SignedMessage {{ .. }} "),
            AgentMessage { .. } => write!(f, "AgentMessage {{ .. }} "),
            ConsensusRequest { .. } => write!(f, "ConsensusRequest {{ .. }} "),
            DagConsensusRequest { .. } => write!(f, "DagConsensusRequest {{ .. }} "),
            DagConsensusResponse { .. } => write!(f, "DagConsensusResponse {{ .. }} "),
            InitBenchmarking { .. } => write!(f, "InitBenchmarking"),
            CompleteRound { .. } => write!(f, "CompleteRound"),
            BenchmarkStats { .. } => write!(f, "BenchmarkStats"),
            BatchedConsensusRequest { .. } => write!(f, "BatchedConsensusRequest"),
            BatchedConsensusResponse { .. } => write!(f, "BatchedConsensusResponse"),
            RoutingTable { .. } => write!(f, "RoutingTable"),
        }
    }
}
