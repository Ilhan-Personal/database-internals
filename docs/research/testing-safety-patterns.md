# Testing & Safety Patterns for Rust Data Structures

**Focus:** B-trees, logging systems, and concurrent data structures  
**Audience:** Educational database internals project  
**Date:** 2026-05-16

---

## Testing Patterns Deep Dive

### 1. Unit Testing Data Structures

#### Test fixtures and helpers:

```rust
// src/test_utils.rs (conditionally compiled)

#[cfg(test)]
pub mod fixtures {
    use crate::index::BTree;

    /// Create a sample B-tree with predictable data
    pub fn sample_btree_100_items() -> BTree<i32, String> {
        let mut tree = BTree::new();
        for i in 0..100 {
            tree.insert(i, format!("value_{}", i));
        }
        tree
    }

    /// Create a B-tree with sequential inserts to stress rebalancing
    pub fn btree_sequential_inserts(count: usize) -> BTree<i32, i32> {
        let mut tree = BTree::new();
        for i in 0..count as i32 {
            tree.insert(i, i);
        }
        tree
    }

    /// Create a B-tree with random order inserts
    pub fn btree_random_inserts(count: usize, seed: u64) -> BTree<i32, i32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut tree = BTree::new();
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let mut rng_state = hasher.finish();

        for _ in 0..count {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            tree.insert((rng_state % 10000) as i32, (rng_state % 10000) as i32);
        }
        tree
    }
}

#[cfg(test)]
pub mod assertions {
    use crate::index::BTree;

    /// Verify all invariants of a B-tree
    pub fn assert_btree_valid<K: Ord + std::fmt::Debug, V>(tree: &BTree<K, V>) {
        tree.validate()
            .expect("B-tree invariants violated")
    }

    /// Verify B-tree height is balanced
    pub fn assert_tree_height_balanced<K: Ord, V>(tree: &BTree<K, V>, max_height: usize) {
        let height = tree.height();
        assert!(
            height <= max_height,
            "Tree height {} exceeds expected max {}",
            height,
            max_height
        );
    }

    /// Verify ordering by checking iterator output
    pub fn assert_tree_iteration_ordered<K: Ord + Clone, V>(tree: &BTree<K, V>) {
        let keys: Vec<K> = tree.iter().map(|(k, _)| k.clone()).collect();
        for window in keys.windows(2) {
            assert!(
                window[0] < window[1],
                "Key ordering violated: {:?} not < {:?}",
                window[0],
                window[1]
            );
        }
    }
}
```

#### Comprehensive unit test examples:

```rust
// In src/index/btree.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    mod construction {
        use super::*;

        #[test]
        fn new_tree_is_empty() {
            let tree: BTree<i32, i32> = BTree::new();
            assert_eq!(tree.len(), 0);
            assert!(tree.is_empty());
        }

        #[test]
        fn new_tree_has_height_zero() {
            let tree: BTree<i32, i32> = BTree::new();
            assert_eq!(tree.height(), 0);
        }
    }

    mod insertion {
        use super::*;

        #[test]
        fn insert_single_item() {
            let mut tree = BTree::new();
            let old = tree.insert(10, "a");
            assert_eq!(old, None);
            assert_eq!(tree.len(), 1);
        }

        #[test]
        fn insert_duplicate_returns_old_value() {
            let mut tree = BTree::new();
            tree.insert(10, "first");
            let old = tree.insert(10, "second");
            assert_eq!(old, Some("first"));
            assert_eq!(tree.len(), 1); // Still one item
        }

        #[test]
        fn insert_multiple_items() {
            let mut tree = BTree::new();
            for i in 0..50 {
                tree.insert(i, i);
            }
            assert_eq!(tree.len(), 50);
        }

        #[test]
        fn insert_maintains_ordering() {
            let mut tree = BTree::new();
            let items = vec![50, 25, 75, 10, 30, 60, 90];
            for item in &items {
                tree.insert(*item, *item);
            }

            let ordered: Vec<_> = tree.iter().map(|(k, _)| *k).collect();
            assert_eq!(ordered, vec![10, 25, 30, 50, 60, 75, 90]);
        }

        #[test]
        fn insert_triggers_node_split() {
            // This test verifies that inserting beyond node capacity causes splits
            let mut tree = BTree::new();
            
            // Tree order=3 means max 5 keys per node
            // Insert enough to force splits
            for i in 0..10 {
                tree.insert(i, i);
            }

            // Verify tree is balanced after splits
            assertions::assert_tree_height_balanced(&tree, 3);
            assertions::assert_btree_valid(&tree);
        }
    }

    mod deletion {
        use super::*;

        #[test]
        fn delete_empty_tree_returns_none() {
            let mut tree: BTree<i32, i32> = BTree::new();
            assert_eq!(tree.remove(&10), None);
        }

        #[test]
        fn delete_single_item() {
            let mut tree = BTree::new();
            tree.insert(10, "value");
            let removed = tree.remove(&10);
            assert_eq!(removed, Some("value"));
            assert_eq!(tree.len(), 0);
        }

        #[test]
        fn delete_maintains_ordering() {
            let mut tree = BTree::new();
            for i in 0..50 {
                tree.insert(i, i);
            }

            // Delete some items
            for i in (0..50).step_by(2) {
                tree.remove(&i);
            }

            // Verify remaining items are still ordered
            let remaining: Vec<_> = tree.iter().map(|(k, _)| *k).collect();
            for window in remaining.windows(2) {
                assert!(window[0] < window[1]);
            }
        }
    }

    mod lookup {
        use super::*;

        #[test]
        fn get_existing_key() {
            let mut tree = BTree::new();
            tree.insert(10, "found");
            assert_eq!(tree.get(&10), Some(&"found"));
        }

        #[test]
        fn get_missing_key_returns_none() {
            let mut tree = BTree::new();
            tree.insert(10, "value");
            assert_eq!(tree.get(&20), None);
        }

        #[test]
        fn get_from_populated_tree() {
            let tree = fixtures::sample_btree_100_items();
            assert_eq!(tree.get(&0), Some(&"value_0".to_string()));
            assert_eq!(tree.get(&50), Some(&"value_50".to_string()));
            assert_eq!(tree.get(&99), Some(&"value_99".to_string()));
            assert_eq!(tree.get(&100), None);
        }
    }

    mod iteration {
        use super::*;

        #[test]
        fn iterate_empty_tree() {
            let tree: BTree<i32, i32> = BTree::new();
            assert_eq!(tree.iter().count(), 0);
        }

        #[test]
        fn iterate_in_order() {
            let tree = fixtures::sample_btree_100_items();
            let keys: Vec<_> = tree.iter().map(|(k, _)| *k).collect();
            
            assert_eq!(keys.len(), 100);
            for (i, &key) in keys.iter().enumerate() {
                assert_eq!(key, i as i32);
            }
        }

        #[test]
        fn iterate_after_insertions_and_deletions() {
            let mut tree = BTree::new();
            for i in 0..100 {
                tree.insert(i, i);
            }
            for i in (0..100).step_by(2) {
                tree.remove(&i);
            }

            let remaining: Vec<_> = tree.iter().map(|(k, _)| *k).collect();
            let expected: Vec<_> = (0..100).filter(|i| i % 2 != 0).collect();
            assert_eq!(remaining, expected);
        }
    }

    mod range_queries {
        use super::*;

        #[test]
        fn range_query_basic() {
            let tree = fixtures::sample_btree_100_items();
            let results: Vec<_> = tree.range(20..30).collect();
            
            assert_eq!(results.len(), 10);
            assert!(results.iter().all(|(k, _)| k >= &20 && k < &30));
        }

        #[test]
        fn range_query_inclusive_exclusive() {
            let tree = fixtures::sample_btree_100_items();
            let results: Vec<_> = tree.range(20..=30).collect();
            
            assert_eq!(results.len(), 11);
            assert!(results.last().map(|(k, _)| k) == Some(&30));
        }

        #[test]
        fn range_query_partial_ranges() {
            let tree = fixtures::sample_btree_100_items();
            
            let from_10: Vec<_> = tree.range(10..).collect();
            assert_eq!(from_10.len(), 90);
            
            let to_50: Vec<_> = tree.range(..50).collect();
            assert_eq!(to_50.len(), 50);
        }
    }

    mod invariants {
        use super::*;

        #[test]
        fn validate_empty_tree() {
            let tree: BTree<i32, i32> = BTree::new();
            assert!(tree.validate().is_ok());
        }

        #[test]
        fn validate_populated_tree() {
            let tree = fixtures::sample_btree_100_items();
            assertions::assert_btree_valid(&tree);
        }

        #[test]
        fn validate_after_operations() {
            let mut tree = BTree::new();
            for i in 0..100 {
                tree.insert(i, i);
                assertions::assert_btree_valid(&tree);
            }
            for i in (0..50) {
                tree.remove(&i);
                assertions::assert_btree_valid(&tree);
            }
        }
    }
}
```

### 2. Property-Based Testing

Essential for data structures; catches edge cases systematically:

```rust
// In src/index/btree.rs

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Strategy: generate vectors of integers
    fn arb_keys() -> impl Strategy<Value = Vec<i32>> {
        prop::collection::vec(0i32..10000, 0..200)
    }

    proptest! {
        /// Property: After inserting all keys, tree contains exactly those keys
        #[test]
        fn prop_insert_all_keys_retrievable(mut keys in arb_keys()) {
            keys.sort();
            keys.dedup();

            let mut tree = BTree::new();
            for key in &keys {
                tree.insert(*key, *key);
            }

            // Every inserted key should be found
            for key in &keys {
                prop_assert_eq!(tree.get(key), Some(key));
            }

            // Tree size should match unique keys
            prop_assert_eq!(tree.len(), keys.len());
        }

        /// Property: Tree iteration yields keys in sorted order
        #[test]
        fn prop_iteration_maintains_order(keys in arb_keys()) {
            let mut tree = BTree::new();
            for key in &keys {
                tree.insert(*key, *key);
            }

            let iterated: Vec<_> = tree.iter().map(|(k, _)| *k).collect();
            let mut sorted = keys.clone();
            sorted.sort();
            sorted.dedup();

            prop_assert_eq!(iterated, sorted);
        }

        /// Property: Deleting all inserted keys leaves empty tree
        #[test]
        fn prop_delete_all_empties_tree(keys in arb_keys()) {
            let mut tree = BTree::new();
            for key in &keys {
                tree.insert(*key, *key);
            }

            for key in &keys {
                tree.remove(key);
            }

            prop_assert_eq!(tree.len(), 0);
        }

        /// Property: Range query results are within bounds and sorted
        #[test]
        fn prop_range_query_correctness(
            keys in arb_keys(),
            start in 0i32..5000,
            end in 5000i32..10000,
        ) {
            let mut tree = BTree::new();
            for key in &keys {
                tree.insert(*key, *key);
            }

            let results: Vec<_> = tree.range(start..end).collect();

            // All results must be in range
            prop_assert!(results.iter().all(|(k, _)| *k >= start && *k < end));

            // Results must be sorted
            for window in results.windows(2) {
                prop_assert!(window[0].0 < window[1].0);
            }

            // Results must match keys in range
            let expected: Vec<_> = keys.iter()
                .filter(|k| **k >= start && **k < end)
                .collect();
            prop_assert_eq!(results.len(), expected.len());
        }

        /// Property: Insert and delete sequences maintain validity
        #[test]
        fn prop_insert_delete_sequence(
            operations in prop::collection::vec(
                (0i32..1000, prop_oneof![Just(true), Just(false)]),
                0..300
            )
        ) {
            let mut tree = BTree::new();
            let mut reference = std::collections::BTreeMap::new();

            for (key, is_insert) in operations {
                if is_insert {
                    tree.insert(key, key);
                    reference.insert(key, key);
                } else {
                    tree.remove(&key);
                    reference.remove(&key);
                }
                
                // Verify validity after each operation
                tree.validate()
                    .expect("Tree invariants violated after operation");
            }

            // Final state must match reference
            let tree_keys: Vec<_> = tree.iter().map(|(k, _)| *k).collect();
            let ref_keys: Vec<_> = reference.keys().cloned().collect();
            prop_assert_eq!(tree_keys, ref_keys);
        }
    }
}
```

### 3. Integration Testing

Test across module boundaries:

```rust
// tests/integration_btree.rs

use my_db::index::BTree;
use my_db::storage::{Storage, FileStorage};
use tempfile::TempDir;

#[test]
fn integration_btree_persists_to_storage() {
    let dir = TempDir::new().unwrap();
    let mut storage = FileStorage::new(dir.path()).unwrap();

    // Create and populate B-tree
    let mut tree = BTree::new();
    for i in 0..100 {
        tree.insert(i, format!("value_{}", i));
    }

    // Simulate persistence: serialize to page
    let serialized = bincode::serialize(&tree).unwrap();
    let page = my_db::storage::Page::from_data(&serialized);
    storage.write_page(0, &page).unwrap();

    // Reload from storage
    let page = storage.read_page(0).unwrap();
    let reloaded: BTree<i32, String> = bincode::deserialize(page.data()).unwrap();

    // Verify
    assert_eq!(reloaded.len(), 100);
    assert_eq!(reloaded.get(&50), Some(&"value_50".to_string()));
    
    // Verify structure integrity
    reloaded.validate().unwrap();
}

#[test]
fn integration_btree_survives_multiple_persists() {
    let dir = TempDir::new().unwrap();
    let mut storage = FileStorage::new(dir.path()).unwrap();

    let mut tree = BTree::new();
    
    // First persist
    for i in 0..50 {
        tree.insert(i, i);
    }
    let page1 = my_db::storage::Page::from_data(&bincode::serialize(&tree).unwrap());
    storage.write_page(0, &page1).unwrap();

    // Reload and add more
    let page = storage.read_page(0).unwrap();
    let mut tree: BTree<i32, i32> = bincode::deserialize(page.data()).unwrap();
    
    for i in 50..100 {
        tree.insert(i, i);
    }
    
    // Second persist
    let page2 = my_db::storage::Page::from_data(&bincode::serialize(&tree).unwrap());
    storage.write_page(1, &page2).unwrap();

    // Final reload and verify
    let page = storage.read_page(1).unwrap();
    let final_tree: BTree<i32, i32> = bincode::deserialize(page.data()).unwrap();
    assert_eq!(final_tree.len(), 100);
    assert_eq!(final_tree.get(&99), Some(&99));
}
```

---

## Memory & Concurrency Safety Patterns

### 1. Initial Single-Threaded Design (Recommended for Phase 1-2)

Use ownership and borrowing exclusively:

```rust
// src/index/btree.rs - Single-threaded version

pub struct BTree<K, V> {
    root: Option<Box<Node<K, V>>>,
    len: usize,
}

impl<K: Ord, V> BTree<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // Mutable borrow ensures exclusive access
        // Compiler prevents concurrent access automatically
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        // Shared borrow allows multiple readers, blocks writers
    }

    pub fn iter(&self) -> Iter<K, V> {
        // Returns iterator with lifetime tied to &self
        Iter { /* ... */ }
    }
}

#[test]
fn single_threaded_safety() {
    let mut tree = BTree::new();
    tree.insert(1, "a");
    
    // This works: mutable borrow for modification
    tree.insert(2, "b");
    
    // This works: shared borrow for reading
    let _ = tree.get(&1);
    
    // This would NOT compile (uncomment to see error):
    // let shared = tree.iter();
    // tree.insert(3, "c");  // Error: cannot borrow tree as mutable while shared borrow exists
}
```

### 2. Thread-Safe Version (for Phase 3+ if concurrent access is added)

Only add when actually needed:

```rust
// src/index/btree_concurrent.rs - Thread-safe version

use parking_lot::RwLock;
use std::sync::Arc;

pub type ConcurrentBTree<K, V> = Arc<RwLock<BTree<K, V>>>;

impl<K: Ord + Send + Sync, V: Send + Sync> BTree<K, V> {
    /// Shared B-tree for multi-threaded access
    pub fn new_concurrent(self) -> ConcurrentBTree<K, V> {
        Arc::new(RwLock::new(self))
    }
}

#[cfg(test)]
mod concurrent_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn concurrent_reads() {
        let tree = Arc::new(RwLock::new(BTree::new()));
        
        // Insert data
        {
            let mut tree = tree.write();
            for i in 0..100 {
                tree.insert(i, i);
            }
        }

        // Spawn read threads
        let handles: Vec<_> = (0..10).map(|thread_id| {
            let tree = Arc::clone(&tree);
            thread::spawn(move || {
                let tree = tree.read();
                for i in 0..100 {
                    assert_eq!(tree.get(&i), Some(&i));
                }
            })
        }).collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn concurrent_read_write() {
        let tree = Arc::new(RwLock::new(BTree::new()));

        let writer = {
            let tree = Arc::clone(&tree);
            thread::spawn(move || {
                for i in 100..200 {
                    let mut tree = tree.write();
                    tree.insert(i, i);
                }
            })
        };

        let readers: Vec<_> = (0..5).map(|_| {
            let tree = Arc::clone(&tree);
            thread::spawn(move || {
                for i in 100..200 {
                    let tree = tree.read();
                    if let Some(&v) = tree.get(&i) {
                        // Verify read
                    }
                }
            })
        }).collect();

        writer.join().unwrap();
        for handle in readers {
            handle.join().unwrap();
        }
    }
}
```

### 3. Unsafe Code Boundaries (Rarely needed for educational B-tree)

If low-level optimizations are added, isolate unsafe code:

```rust
// src/index/node.rs - Minimal unsafe usage

impl<K, V> Node<K, V> {
    /// Safe public API
    pub fn child_at(&self, index: usize) -> Option<&Node<K, V>> {
        self.children.get(index).and_then(|c| c.as_ref())
    }

    /// Internal: Only use unsafe when needed for performance (e.g., tight loops)
    #[inline(always)]
    pub(crate) fn keys_slice(&self) -> &[K] {
        // SAFETY: self.keys is always valid; we return a safe slice
        unsafe {
            std::slice::from_raw_parts(
                self.keys.as_ptr(),
                self.keys.len(),
            )
        }
    }

    /// Validate unsafe operations are correct
    #[test]
    fn test_keys_slice_safety() {
        let node = Node::new(true);
        let slice = node.keys_slice();
        // Ensure slice is valid and correct
        assert_eq!(slice.len(), 0);
    }
}
```

---

## Error Handling Patterns

### For database structures:

```rust
// src/error.rs

use thiserror::Error;
use std::io;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Page {id} not found")]
    PageNotFound { id: u32 },

    #[error("Serialization failed: {0}")]
    Serialization(String),

    #[error("Tree invariant violated: {0}")]
    InvariantViolation(String),
}

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("Key {key} already exists (expected: insert only)")]
    DuplicateKey { key: String },

    #[error("Tree is corrupted: {reason}")]
    Corrupted { reason: String },
}

pub type Result<T> = std::result::Result<T, StorageError>;
pub type IndexResult<T> = std::result::Result<T, IndexError>;

// Usage in code:
impl BTree<K, V> {
    pub fn validate(&self) -> IndexResult<()> {
        if !self.is_valid() {
            return Err(IndexError::Corrupted {
                reason: "Keys not sorted".to_string(),
            });
        }
        Ok(())
    }
}

// Test error handling:
#[test]
fn test_error_propagation() {
    let result: IndexResult<()> = Err(IndexError::Corrupted {
        reason: "test".to_string(),
    });
    
    assert!(result.is_err());
    assert!(format!("{}", result.unwrap_err()).contains("corrupted"));
}
```

---

## Recommended Test Coverage Strategy

For educational project, aim for:

| Component | Unit Tests | Integration | Property | Coverage |
|-----------|-----------|-------------|----------|----------|
| Storage layer | ✓ Read/write basics | ✓ Persistence | ✗ | >80% |
| B-tree node | ✓ Insert/delete/search | ✗ | ✓ Invariants | >85% |
| B-tree full | ✓ All operations | ✓ With storage | ✓ Insert/delete/range | >90% |
| Query parser | ✓ Valid/invalid syntax | ✓ End-to-end | ✗ | >80% |
| Executor | ✓ Each operation | ✓ Full queries | ✗ | >75% |

Run tests with: `cargo test --lib && cargo test --test '*' && cargo test --doc`

---

## Safety Checklist

Before considering code complete:

- [ ] All borrowing rules enforced by compiler (no unsafe needed yet)
- [ ] All public functions have error handling (Result types)
- [ ] All tree operations validate invariants (in tests + optional debug assertions)
- [ ] Memory layout is reasonable (no excessive allocations)
- [ ] Examples in docs are correct and run as doc tests
- [ ] Integration tests verify cross-module behavior
- [ ] Property tests catch edge cases
- [ ] No panics in library code (return Result instead)
- [ ] Unsafe code (if any) is documented and tested

This approach ensures the code is both safe and educational.
