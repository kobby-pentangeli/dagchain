use crate::{
    account::AccountStateChoice,
    config::ConsensusConfig,
    network::{CommonConsensusNetwork, ConsensusNetwork},
    transaction::Transaction,
    tree::HashTreeNode,
    AccountConflictSet, Consensus, ConsensusStatus,
};
use crypto::hash::Hash;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

pub struct QuantumConsensus {
    conflict_set: Arc<RwLock<AccountConflictSet>>,
    choice: Arc<RwLock<HashMap<Hash, Hash>>>,
    // network: Box<dyn ConsensusNetwork>,
    config: ConsensusConfig,
}

impl QuantumConsensus {
    /// Check if a conflict set exists for an account state
    fn has_conflicts(&self, state: &AccountStateChoice) -> bool {
        self.conflict_set
            .write()
            .unwrap()
            .get(&state.account_state_id)
            .is_some()
    }
}

impl Consensus for QuantumConsensus {
    fn new(config: ConsensusConfig) -> Self
    where
        Self: Sized,
    {
        Self {
            conflict_set: Arc::new(RwLock::new(HashMap::new())),
            choice: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    fn query(&self, state: &AccountStateChoice) -> &Self
    where
        Self: Sized,
    {
        {
            let mut cs = self.conflict_set.write().unwrap();
            if let Some(set) = cs.get_mut(&state.account_state_id) {
                set.insert(state.tx.get_tx_id());
            } else {
                let mut set: HashSet<Hash> = HashSet::new();
                set.insert(state.tx.get_tx_id());
                cs.insert(state.account_state_id, set);
            }
        }
        self
    }

    fn send_consensus_requests<T, N>(
        &self,
        state: &AccountStateChoice,
        tx: &Transaction,
        network: &mut T,
        common_network: &mut N,
        count: usize,
    ) where
        T: ConsensusNetwork,
        N: CommonConsensusNetwork,
    {
        self.query(state);

        log::info!("PRINT: fire_consensus: #3");
        network.send_dag_queries(self.config.k, tx, state, common_network, count);
    }

    fn complete_dag_consensus(
        &self,
        _acceptance: usize,
        _state: &AccountStateChoice,
        _tree: &mut HashTreeNode,
    ) -> ConsensusStatus {
        unimplemented!()
    }

    fn fire_consensus<T, N>(
        &mut self,
        state: &AccountStateChoice,
        network: &mut T,
        common_network: &mut N,
        _tree: Option<&mut HashTreeNode>,
    ) -> ConsensusStatus
    where
        T: ConsensusNetwork,
        N: CommonConsensusNetwork,
    {
        let exists = self.has_conflicts(state);
        self.query(state);
        if exists {
            return ConsensusStatus::InProgress;
        }
        let mut confidence: HashMap<Hash, u64> = HashMap::new();
        let mut choice = state.tx.get_tx_id();
        {
            self.choice
                .write()
                .unwrap()
                .insert(state.account_state_id, choice);
        }
        let mut last_choice = state.tx.get_tx_id();
        let mut choice_count: u64 = 0;
        loop {
            log::info!("PRINT: choice_count: {:?}", choice_count);
            let acceptance = network.query(self.config.k, &state, common_network);
            log::info!("PRINT: acceptance: {:?}\n", acceptance);

            let cs = self
                .conflict_set
                .read()
                .unwrap()
                .get(&state.account_state_id)
                .unwrap()
                .clone();
            for set_id in &cs {
                log::info!("PRINT: set_id: {:?}", set_id);
                if let Some(p) = acceptance.get(set_id) {
                    log::info!("PRINT:# set_id: {:?} [{:?}]", set_id, p);
                    if self.config.threshold(*p) {
                        *confidence.entry(*set_id).or_insert(1) += 1;
                        let iterated_confidence_count = confidence.get(set_id);
                        let current_confidence_count = confidence.get(&choice);
                        if iterated_confidence_count.is_some()
                            && current_confidence_count.is_some()
                            && (iterated_confidence_count.unwrap()
                                > current_confidence_count.unwrap())
                        {
                            choice = *set_id;
                            self.choice
                                .write()
                                .unwrap()
                                .entry(state.account_state_id)
                                .or_insert(choice);
                        }
                        if last_choice != *set_id {
                            last_choice = *set_id;
                            choice_count = 0;
                        } else {
                            choice_count += 1;
                            if choice_count > self.config.beta {
                                return ConsensusStatus::Accept(choice);
                            }
                        }
                    }
                }
            }
            //break;
        }
        //ConsensusStatus::Reject
    }

    fn on_query(&self, state: &AccountStateChoice) -> (Hash, bool) {
        let exists = if let Some(set) = self
            .conflict_set
            .write()
            .unwrap()
            .get(&state.account_state_id)
        {
            set.get(&state.tx.get_tx_id()).is_some()
        } else {
            false
        };
        if let Some(choice) = self.choice.write().unwrap().get(&state.account_state_id) {
            return (*choice, exists);
        }
        (state.tx.get_tx_id(), exists)
    }

    fn target_count(&self) -> usize {
        self.config.k as usize
    }
}
