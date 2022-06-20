use serde::{Deserialize, Serialize};
use structopt::StructOpt;

/// Consensus parameters
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
pub struct ConsensusConfig {
    #[structopt(short, long, default_value = "0.6")]
    pub alpha: f64,
    #[structopt(short, long, default_value = "2")]
    pub beta: u64,
    #[structopt(short, long, default_value = "2")]
    pub beta2: u64,
    #[structopt(short, long, default_value = "10")]
    pub k: u64,
    #[structopt(skip)]
    pub quantum: bool,
    #[structopt(short, long, default_value = "40")]
    pub max_batch_size: usize,
    #[structopt(short, long, default_value = "10")]
    pub max_batch_interval: f32,
}

impl ConsensusConfig {
    /// Initialize config with specific params
    pub fn new(alpha: f64, beta: u64, beta2: u64, k: u64) -> Self {
        Self {
            alpha,
            beta,
            beta2,
            k,
            quantum: false,
            max_batch_size: 40,
            max_batch_interval: 2.0,
        }
    }

    /// Change consensus to Quantum by default
    pub fn set_quantum_consensus(&mut self) {
        self.quantum = true;
    }

    /// Check threshold for coefficients
    pub fn threshold(&self, param: u64) -> bool {
        param as f64 > self.alpha * self.k as f64
    }
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            alpha: 0.66,
            beta: 2,
            beta2: 2,
            k: 10,
            quantum: false,
            max_batch_size: 40,
            max_batch_interval: 2.0,
        }
    }
}
