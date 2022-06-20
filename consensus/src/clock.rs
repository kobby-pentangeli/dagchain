use crypto::hash::Hash;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::{Entry, HashMap};

/// HVC - Basic representation of a hierarchical vector clock
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Hvc {
    vector: HashMap<Hash, u64>,
    hierarchical_order: LogicalClock,
}

impl Hvc {
    pub fn new() -> Self {
        Hvc {
            vector: HashMap::new(),
            hierarchical_order: LogicalClock::new(),
        }
    }

    pub fn order(&mut self) -> &mut LogicalClock {
        &mut self.hierarchical_order
    }

    pub fn increment(&mut self, node_id: Hash) {
        self.vector
            .entry(node_id)
            .and_modify(|e| *e += 1)
            .or_insert(1);
    }

    pub fn happened_before(&self, other: &Self) -> bool {
        use std::cmp::Ordering;

        for (key, &self_clock) in self.vector.iter() {
            let other_clock = *other.vector.get(key).unwrap_or(&0);
            match self_clock.cmp(&other_clock) {
                Ordering::Greater => return false,
                Ordering::Less => return true,
                _ => (),
            }
        }

        for (key, &other_clock) in other.vector.iter() {
            let self_clock = *self.vector.get(&key).unwrap_or(&0);
            match self_clock.cmp(&other_clock) {
                Ordering::Greater => return false,
                Ordering::Less => return true,
                _ => (),
            }
        }

        false
    }

    pub fn merge(&self, other: &Self) -> Self {
        let mut merge_res = self.vector.clone();

        for (key, &other_clock) in other.vector.iter() {
            match merge_res.entry(*key) {
                Entry::Vacant(e) => {
                    e.insert(other_clock);
                }
                Entry::Occupied(mut e) => {
                    let self_clock = *e.get();
                    e.insert(self_clock + other_clock);
                }
            }
        }

        Self {
            vector: merge_res,
            hierarchical_order: LogicalClock::new(),
        }
    }

    pub fn are_concurrent(&self, other: &Self) -> bool {
        !(self.happened_before(other) || other.happened_before(self))
    }
}

/// Logical clock for HVC
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LogicalClock(u64);

impl LogicalClock {
    /// Initialize LogicalClock
    pub fn new() -> Self {
        Self(0)
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    /// Get current clock state
    pub fn get(&self) -> u64 {
        self.0
    }

    pub fn is_equal(&self, other: Self) -> bool {
        self.0 == other.0
    }

    pub fn is_less(&self, other: Self) -> bool {
        self.0 < other.0
    }

    pub fn is_greater(&self, other: Self) -> bool {
        self.0 > other.0
    }
}

#[test]
fn test_hvc_init() {
    let hash_a = Hash::new("A".as_bytes());
    let hash_b = Hash::new("B".as_bytes());
    let hash_c = Hash::new("C".as_bytes());

    let mut hvc = Hvc::new();
    hvc.increment(hash_a);
    hvc.increment(hash_b);
    hvc.increment(hash_a);
    hvc.increment(hash_c);

    assert_eq!(hvc.vector.get(&hash_a).unwrap(), &2);
    assert_eq!(hvc.vector.get(&hash_b).unwrap(), &1);
    assert_eq!(hvc.vector.get(&hash_c).unwrap(), &1);
}

#[test]
fn test_hvc_merge() {
    let hash_a = Hash::new("A".as_bytes());
    let hash_b = Hash::new("B".as_bytes());

    let mut hvc1 = Hvc::new();
    hvc1.increment(hash_a);
    hvc1.increment(hash_a);
    hvc1.increment(hash_b);

    let mut hvc2 = Hvc::new();
    hvc2.increment(hash_a);
    hvc2.increment(hash_b);
    hvc2.increment(hash_a);

    let hvc3 = hvc1.merge(&hvc2);

    assert_eq!(hvc3.vector.get(&hash_a).unwrap(), &4);
    assert_eq!(hvc3.vector.get(&hash_b).unwrap(), &2);
}

#[test]
fn test_hvc_happened_before() {
    let hash_a = Hash::new("A".as_bytes());
    let hash_b = Hash::new("B".as_bytes());
    let hash_c = Hash::new("C".as_bytes());

    // case 0: hvc1 happened before hvc2

    // [2, 3, 2]
    let mut hvc1 = Hvc::new();
    hvc1.increment(hash_a);
    hvc1.increment(hash_a);
    hvc1.increment(hash_b);
    hvc1.increment(hash_b);
    hvc1.increment(hash_b);
    hvc1.increment(hash_c);
    hvc1.increment(hash_c);

    // [2, 4, 2]
    let mut hvc2 = Hvc::new();
    hvc2.increment(hash_a);
    hvc2.increment(hash_a);
    hvc2.increment(hash_b);
    hvc2.increment(hash_b);
    hvc2.increment(hash_b);
    hvc2.increment(hash_b);
    hvc2.increment(hash_c);
    hvc2.increment(hash_c);

    assert!(hvc1.happened_before(&hvc2));
    assert!(!hvc2.happened_before(&hvc1));
}
