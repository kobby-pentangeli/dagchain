use crypto::hash::Hash;
use std::collections::HashMap;

/// Hash Tree Node
/// Basic representation of a consensus tree structure
pub type HashTreeNode = HashMap<Hash, (Hash, TreeNode)>;

#[derive(Clone, Debug, PartialEq)]
pub struct TreeNode {
    pub node: Hash,
    pub confidence: u64,
    pub preferred: Hash,
    pub last: Hash,
    pub count: u64,
}

impl TreeNode {
    /// Initialize a TreeNode
    pub fn new(node: Hash) -> Self {
        Self {
            node,
            confidence: 0,
            preferred: node,
            last: node,
            count: 0,
        }
    }

    /// Set preferred value
    pub fn set_preferred(&mut self, p: Hash) -> &mut Self {
        self.preferred = p;
        self
    }

    /// Set last selected value
    pub fn set_last(&mut self, l: Hash) -> &mut Self {
        self.last = l;
        self
    }

    /// Set confidence for TreeNode
    pub fn set_confidence(&mut self, c: u64) -> &mut Self {
        self.confidence = c;
        self
    }

    /// Increment count
    pub fn increment_count(&mut self) -> &mut Self {
        self.count += 1;
        self
    }
}
