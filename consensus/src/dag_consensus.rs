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

pub struct DagConsensus {
    conflict_set: Arc<RwLock<AccountConflictSet>>,
    choice: Arc<RwLock<HashMap<Hash, Hash>>>,
    config: ConsensusConfig,
}

impl Consensus for DagConsensus {
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
            let mut conflict_set = self.conflict_set.write().unwrap();
            if let Some(set) = conflict_set.get_mut(&state.account_state_id) {
                set.insert(state.tx.get_tx_id());
            } else {
                let mut set: HashSet<Hash> = HashSet::new();
                set.insert(state.tx.get_tx_id());
                conflict_set.insert(state.account_state_id, set);
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
        network.send_dag_queries_batched(
            self.config.k,
            tx,
            &state,
            common_network,
            self.config.max_batch_size,
            self.config.max_batch_interval,
            count,
        );
    }

    fn complete_dag_consensus(
        &self,
        acceptance: usize,
        state: &AccountStateChoice,
        tree: &mut HashTreeNode,
    ) -> ConsensusStatus {
        log::info!("ACCEPTANCE: {}", acceptance as u64);
        if self.config.threshold(acceptance as u64) {
            log::info!("PRINT: fire_consensus: #5");
            {
                let mut store = self.choice.write().unwrap();
                if store.get(&state.account_state_id).is_some() {
                    log::error!("REJECT: account state doesn't exist");
                    return ConsensusStatus::Reject;
                }
                store.insert(state.account_state_id, state.tx.get_tx_id());
            }

            let mut parent_hash = state.tx.parent;
            log::info!(
                "PRINT: fire_consensus: #6 {:#?} {:#?}",
                tree.get(&parent_hash),
                parent_hash
            );
            while let Some(path) = tree.get(&parent_hash) {
                log::info!("PRINT:fire_consensus: #7 parent_hash {:#?}", path.0);
                parent_hash = path.0;
                let mut node = path.clone().1;
                if let Some(preferred_confidence) = tree.get(&node.preferred) {
                    let preferred_confidence = preferred_confidence.clone().1;
                    if node.confidence > preferred_confidence.confidence {
                        node.preferred = node.node;
                    }
                    if node.node != node.last {
                        node.last = node.node;
                        node.count = 0;
                    } else {
                        node.count += 1;
                    }
                }
                let updated_node = (parent_hash, node.clone());
                *tree.entry(parent_hash).or_insert(updated_node) = updated_node.clone();
                if node.confidence > self.config.beta {
                    return ConsensusStatus::Accept(node.node);
                }
                if node.count > self.config.beta2 {
                    return ConsensusStatus::Accept(state.tx.get_tx_id());
                }
            }
            log::error!("REJECT: Reached threshold but not accepted")
        }
        log::error!("REJECT: Request not accepted");
        ConsensusStatus::Reject
    }

    fn fire_consensus<T, N>(
        &mut self,
        state: &AccountStateChoice,
        network: &mut T,
        common_network: &mut N,
        tree: Option<&mut HashTreeNode>,
    ) -> ConsensusStatus
    where
        T: ConsensusNetwork,
        N: CommonConsensusNetwork,
    {
        self.query(state);
        let tree = tree.unwrap();

        log::info!("PRINT: fire_consensus: #3");
        let p = network.dag_query(self.config.k, &state, common_network);
        log::info!("PRINT: fire_consensus: #4 {:?}", p);
        if self.config.threshold(p) {
            log::info!("PRINT: fire_consensus: #5");
            {
                let mut store = self.choice.write().unwrap();
                if store.get(&state.account_state_id).is_some() {
                    return ConsensusStatus::Reject;
                }
                store.insert(state.account_state_id, state.tx.get_tx_id());
            }

            let mut parent_hash = state.tx.parent;
            // Fetch Tree
            log::info!(
                "PRINT: fire_consensus: #6 {:#?} {:#?}",
                tree.get(&parent_hash),
                parent_hash
            );
            while let Some(path) = tree.get(&parent_hash) {
                log::info!("PRINT:fire_consensus: #7 parent_hash {:#?}", path.0);
                parent_hash = path.0;
                let mut node = path.clone().1;
                if let Some(preferred_confidence) = tree.get(&node.preferred) {
                    let preferred_confidence = preferred_confidence.clone().1;
                    // Compare Confidence Tree
                    if node.confidence > preferred_confidence.confidence {
                        node.preferred = node.node;
                    }
                    if node.node != node.last {
                        node.last = node.node;
                        node.count = 0;
                    } else {
                        node.count += 1;
                    }
                }
                // Update Tree Node state
                let updated_node = (parent_hash, node.clone());
                *tree.entry(parent_hash).or_insert(updated_node) = updated_node.clone();
                // Check early commitment
                if node.confidence > self.config.beta {
                    return ConsensusStatus::Accept(node.node);
                }
                // Check consecutive counter commitment
                if node.count > self.config.beta2 {
                    return ConsensusStatus::Accept(state.tx.get_tx_id());
                }
            }
        }
        // If request was not accepted return Reject
        ConsensusStatus::Reject
    }

    fn on_query(&self, state: &AccountStateChoice) -> (Hash, bool) {
        log::info!("PRINT: on_query: {:?}", state);
        let exists = if let Some(set) = self
            .conflict_set
            .write()
            .unwrap()
            .get(&state.account_state_id)
        {
            log::info!("PRINT: on_query: is_some {:?}", state);
            set.get(&state.tx.get_tx_id()).is_some()
        } else {
            false
        };
        log::info!("PRINT: on_query: exists {:?}", exists);
        if let Some(choice) = self.choice.write().unwrap().get(&state.account_state_id) {
            return (*choice, exists);
        }
        (state.tx.get_tx_id(), exists)
    }

    fn target_count(&self) -> usize {
        self.config.k as usize
    }
}
