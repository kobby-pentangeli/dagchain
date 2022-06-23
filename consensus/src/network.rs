use crate::{account::AccountStateChoice, transaction::Transaction};
use crypto::hash::Hash;
use std::collections::HashMap;

pub trait CommonConsensusNetwork {
    fn get_nodes_except_one(&self, k: u64, node_id: Hash) -> Vec<Hash>;
}

pub trait ConsensusNetwork {
    fn get_sample_network<T: CommonConsensusNetwork>(
        &self,
        k: u64,
        current_node: Hash,
        network: &T,
    ) -> Vec<Hash>;

    fn request_consensus(&mut self, node_id: Hash, data: &AccountStateChoice) -> Hash;

    fn request_dag_consensus(&self, node_id: Hash, data: &AccountStateChoice) -> bool;

    fn send_dag_consensus_request(
        &mut self,
        node_id: Hash,
        data: &AccountStateChoice,
        tx: &Transaction,
        count: usize,
    );

    fn add_outgoing_dag_consensus_request(
        &mut self,
        node_id: Hash,
        _data: &AccountStateChoice,
        tx: &Transaction,
        count: usize,
    );

    fn accept_incoming_consensus_response(
        &mut self,
        node_id: Hash,
        data: Hash,
        accepted: bool,
    ) -> (usize, usize);

    fn remove_outgoing_dag_transaction(&mut self, tx_id: Hash) -> Transaction;

    fn get_node_id(&self) -> Hash;

    fn query<T: CommonConsensusNetwork>(
        &mut self,
        k: u64,
        data: &AccountStateChoice,
        network: &T,
    ) -> HashMap<Hash, u64> {
        let nodes = self.get_sample_network(k, self.get_node_id(), network);
        log::info!("PRINT: get_sample_network {:?}", nodes);
        let mut query_result: HashMap<Hash, u64> = HashMap::new();
        for node_id in nodes {
            let choice = self.request_consensus(node_id, data);
            *query_result.entry(choice).or_insert(1) += 1;
        }
        query_result
    }

    fn send_dag_queries<N: CommonConsensusNetwork>(
        &mut self,
        k: u64,
        tx: &Transaction,
        data: &AccountStateChoice,
        network: &N,
        count: usize,
    ) {
        let nodes = self.get_sample_network(k, self.get_node_id(), network);
        for node_id in nodes {
            self.add_outgoing_dag_consensus_request(node_id, data, tx, count);
            self.send_dag_consensus_request(node_id, data, tx, count);
        }
    }

    fn send_dag_queries_batched<N: CommonConsensusNetwork>(
        &mut self,
        k: u64,
        tx: &Transaction,
        data: &AccountStateChoice,
        network: &N,
        max_batch_size: usize,
        max_batch_interval: f32,
        count: usize,
    ) {
        self.add_transaction_to_batch(
            k,
            tx,
            data,
            network,
            max_batch_size,
            max_batch_interval,
            count,
        );
    }

    fn add_transaction_to_batch<N: CommonConsensusNetwork>(
        &mut self,
        k: u64,
        tx: &Transaction,
        data: &AccountStateChoice,
        network: &N,
        max_batch_size: usize,
        max_batch_interval: f32,
        count: usize,
    );

    fn dag_query<N: CommonConsensusNetwork>(
        &mut self,
        k: u64,
        data: &AccountStateChoice,
        network: &N,
    ) -> u64 {
        let nodes = self.get_sample_network(k, self.get_node_id(), network);
        log::info!("PRINT: dag_query: {:?}", nodes);
        let mut query_result: u64 = 0;
        for node_id in nodes {
            log::info!("PRINT: dag_query: node_id {:?}", node_id);
            let preferred = self.request_dag_consensus(node_id, data);
            log::info!("PRINT: dag_query: choice {:?}", preferred);
            if preferred {
                query_result += 1;
            }
        }
        query_result
    }
}
