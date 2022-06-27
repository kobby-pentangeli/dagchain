use crypto::hash::Hash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SharedRoutingTable {
    entries: HashMap<Hash, usize>,
}
