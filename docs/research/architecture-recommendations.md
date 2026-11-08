# Database Storage Engine — Concrete Architecture Recommendations

**Date:** 2026-05-16  
**For:** Educational database internals learning project  
**Status:** Applied best practices tailored to project scope

---

## Recommended Project Structure

```
database-engine/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Re-exports + top-level docs
│   │
│   ├── error.rs                    # Centralized error types
│   │
│   ├── storage/                    # Disk I/O & persistence
│   │   ├── mod.rs                  # Public API: Storage, StorageBackend trait
│   │   ├── page.rs                 # Page abstraction (fixed-size blocks)
│   │   ├── file_storage.rs         # File-based implementation
│   │   ├── format.rs               # Serialization helpers
│   │   └── memory_storage.rs       # In-memory backend (for testing)
│   │
│   ├── index/                      # B-tree indexing
│   │   ├── mod.rs                  # Public API: BTree
│   │   ├── btree.rs                # BTree struct and operations
│   │   ├── node.rs                 # Node data structure
│   │   ├── search.rs               # Search algorithms
│   │   └── iter.rs                 # Iterator implementations
│   │
│   ├── buffer/                     # In-memory buffer management
│   │   ├── mod.rs
│   │   └── buffer_pool.rs
│   │
│   ├── query/                      # Query parsing & execution
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   └── executor.rs
│   │
│   └── db.rs                       # Top-level engine: Storage + Index + Query
│
├── examples/
│   ├── basic_kvstore.rs            # Example 1: Simple key-value ops
│   ├── btree_range_queries.rs      # Example 2: B-tree features
│   └── persistence_demo.rs         # Example 3: Disk I/O
│
├── tests/
│   ├── integration_storage.rs      # Storage layer integration
│   ├── integration_btree.rs        # B-tree integration
│   └── integration_full_system.rs  # End-to-end
│
├── benches/
│   └── btree_operations.rs         # Criterion benchmarks
│
└── docs/
    ├── architecture.md             # Design rationale
    ├── research/                   # This research document
    └── api-design.md               # Public API surface
```

---

## Module-by-Module Design

### 1. Error Layer (`src/error.rs`)

Centralize error types for the entire crate. Use `thiserror` for clean derives:

```rust
// src/error.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Page {id} not found")]
    PageNotFound { id: u32 },
    
    #[error("Corrupted page: {reason}")]
    CorruptedPage { reason: String },
}

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("Key not found")]
    KeyNotFound,
    
    #[error("Index corrupted: {0}")]
    Corrupted(String),
}

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;
pub type IndexResult<T> = std::result::Result<T, IndexError>;
pub type QueryResult<T> = std::result::Result<T, QueryError>;
```

### 2. Storage Layer (`src/storage/`)

**Design philosophy:** Abstraction over implementation details. Users interact with the trait, not concrete types.

```rust
// src/storage/mod.rs

pub use self::page::Page;
pub use self::file_storage::FileStorage;

mod page;
mod file_storage;
mod format;

#[cfg(test)]
mod memory_storage;

/// Trait defining how data is persisted
/// Enables testing with in-memory backends
pub trait StorageBackend: Send + Sync {
    fn read_page(&self, id: PageId) -> crate::Result<Page>;
    fn write_page(&mut self, id: PageId, page: &Page) -> crate::Result<()>;
    fn delete_page(&mut self, id: PageId) -> crate::Result<()>;
    fn page_count(&self) -> crate::Result<u32>;
}

/// Main storage engine: manages pages and persistence
pub struct Storage {
    backend: Box<dyn StorageBackend>,
    buffer: crate::buffer::BufferPool,
}

impl Storage {
    pub fn new_file<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let backend = Box::new(FileStorage::new(path)?);
        let buffer = crate::buffer::BufferPool::new(100); // 100 pages
        Ok(Storage { backend, buffer })
    }

    pub fn get_page(&self, id: PageId) -> crate::Result<Page> {
        if let Some(page) = self.buffer.get(id) {
            return Ok(page);
        }
        let page = self.backend.read_page(id)?;
        self.buffer.put(id, page.clone());
        Ok(page)
    }

    pub fn put_page(&mut self, id: PageId, page: Page) -> crate::Result<()> {
        self.backend.write_page(id, &page)?;
        self.buffer.put(id, page);
        Ok(())
    }
}
```

**Key design choices:**
- **StorageBackend trait** allows swapping implementations (file vs. memory for testing)
- **BufferPool** provides caching layer
- **Page abstraction** hides file format details
- **Errors use custom types** (`crate::Result`)

### 3. Index Layer (`src/index/`)

**Design philosophy:** Standard B-tree operations with clear node management.

```rust
// src/index/mod.rs

pub use self::btree::BTree;

mod btree;
mod node;
mod search;
mod iter;

/// Public B-tree API
pub struct BTree<K, V> {
    root: Option<Box<Node<K, V>>>,
    len: usize,
    order: usize,  // B-tree order (determines node branching factor)
}

impl<K: Ord, V> BTree<K, V> {
    pub fn new() -> Self {
        BTree {
            root: None,
            len: 0,
            order: 4,  // Each node can have up to 4 children
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // Insertion logic
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        // Lookup logic
    }

    pub fn range<'a, R>(&'a self, range: R) -> RangeIter<'a, K, V>
    where
        R: std::ops::RangeBounds<K>,
    {
        // Range query logic
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// Validate tree invariants (useful for testing)
    pub fn validate(&self) -> crate::IndexResult<()> {
        // Check ordering, node sizes, height balance
    }
}

impl<K: Ord, V> Default for BTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
```

```rust
// src/index/node.rs

pub struct Node<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
    children: Vec<Box<Node<K, V>>>,  // Empty for leaf nodes
    is_leaf: bool,
}

impl<K: Ord, V> Node<K, V> {
    pub fn new(is_leaf: bool) -> Self {
        Node {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
            is_leaf,
        }
    }

    /// Search within a node's keys
    pub fn find_key(&self, key: &K) -> usize {
        self.keys.binary_search(key).unwrap_or_else(|i| i)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.keys.binary_search(key)
            .ok()
            .and_then(|i| self.values.get(i))
    }

    /// Validation helper for tree invariants
    pub fn validate_node(&self, order: usize) -> crate::IndexResult<()> {
        // Check:
        // - Keys are sorted
        // - Number of keys <= 2*order - 1
        // - For non-leaf: children.len() == keys.len() + 1
        // - All keys are within expected range
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_key_in_leaf() {
        let mut node = Node::new(true);
        node.keys = vec![10, 20, 30];
        assert_eq!(node.find_key(&20), 1);
        assert_eq!(node.find_key(&15), 1);
    }
}
```

**Key design choices:**
- **Order parameter** controls node size (easier to understand than raw page sizes initially)
- **Validate method** helps catch bugs during development
- **Separate node.rs** keeps node logic isolated
- **Unit tests inline** with node logic

### 4. Query Layer (`src/query/`)

```rust
// src/query/mod.rs

pub use self::parser::Query;
pub use self::executor::Executor;

mod parser;
mod executor;

/// Represents a parsed query
#[derive(Debug)]
pub enum Query {
    Get { key: String },
    Set { key: String, value: Vec<u8> },
    Delete { key: String },
    Range { start: String, end: String },
}

impl Query {
    /// Parse from a simple string format: "GET key" or "SET key value"
    pub fn parse(input: &str) -> crate::QueryResult<Self> {
        parser::parse(input)
    }
}

pub struct Executor {
    // Reference to storage and index
}

impl Executor {
    pub fn execute(&self, query: Query) -> crate::QueryResult<QueryResult> {
        executor::execute(self, query)
    }
}

#[derive(Debug)]
pub enum QueryResult {
    Value(Option<Vec<u8>>),
    Ok,
    Rows(Vec<(String, Vec<u8>)>),
}
```

**Key design choices:**
- **Simple enum-based Query** for parsing (not a full AST)
- **Separate executor** from parser
- **Educational focus**: parses subset of SQL, not full standard

### 5. Top-level Engine (`src/db.rs`)

Tie together all layers for end-users:

```rust
// src/db.rs (or src/lib.rs could include this)

use crate::storage::Storage;
use crate::index::BTree;
use crate::query::{Query, Executor};

/// Main database engine
pub struct Engine {
    storage: Storage,
    index: BTree<String, u32>,  // Maps keys to page locations
}

impl Engine {
    pub fn new<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        Ok(Engine {
            storage: Storage::new_file(path)?,
            index: BTree::new(),
        })
    }

    pub fn get(&self, key: &str) -> crate::Result<Option<Vec<u8>>> {
        if let Some(page_id) = self.index.get(key) {
            let page = self.storage.get_page(*page_id)?;
            // Deserialize value from page
            Ok(Some(page.data().to_vec()))
        } else {
            Ok(None)
        }
    }

    pub fn set(&mut self, key: String, value: Vec<u8>) -> crate::Result<()> {
        // Allocate page if needed
        let page_id = self.storage.allocate_page()?;
        let page = Page::from_data(value);
        self.storage.put_page(page_id, page)?;
        self.index.insert(key, page_id);
        Ok(())
    }

    pub fn execute_query(&self, query_str: &str) -> crate::QueryResult<QueryResult> {
        let query = Query::parse(query_str)?;
        let executor = Executor::new(self);
        executor.execute(query)
    }
}
```

---

## Testing Strategy

### Test Organization for This Project:

```
src/
├── storage/
│   ├── file_storage.rs         # Unit tests: test file I/O
│   ├── page.rs                 # Unit tests: page format
│   └── memory_storage.rs       # Test double (in #[cfg(test)])
│
├── index/
│   ├── btree.rs                # Unit tests: insertion, lookup
│   │                            # Property tests: invariants
│   ├── node.rs                 # Unit tests: node operations
│   │
│   └── tests_btree.rs          # Optional: larger unit tests module
│
└── query/
    ├── parser.rs               # Unit tests: parse correctness
    └── executor.rs             # Unit tests: simple execution

tests/
├── integration_storage.rs      # Full storage lifecycle
├── integration_btree.rs        # B-tree across modules
└── integration_end_to_end.rs   # Engine + storage + query
```

### Concrete test examples:

```rust
// In src/index/btree.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_empty_tree() {
        let tree: BTree<i32, i32> = BTree::new();
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.get(&1), None);
    }

    #[test]
    fn test_insert_and_get() {
        let mut tree = BTree::new();
        tree.insert(10, "a");
        tree.insert(20, "b");
        tree.insert(5, "c");

        assert_eq!(tree.get(&10), Some(&"a"));
        assert_eq!(tree.get(&5), Some(&"c"));
        assert_eq!(tree.get(&999), None);
    }

    #[test]
    fn test_tree_maintains_order() {
        let mut tree = BTree::new();
        for i in (0..100).rev() {
            tree.insert(i, i);
        }

        let keys: Vec<_> = tree.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, (0..100).collect::<Vec<_>>());
    }

    #[test]
    fn test_validate_tree_invariants() {
        let mut tree = BTree::new();
        for i in 0..50 {
            tree.insert(i, i);
        }
        assert!(tree.validate().is_ok());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn insert_maintains_order(
            keys in prop::collection::vec(0i32..1000, 0..100)
        ) {
            let mut tree = BTree::new();
            for key in &keys {
                tree.insert(*key, *key);
            }

            let mut sorted = keys.clone();
            sorted.sort();
            sorted.dedup();

            let tree_keys: Vec<_> = tree.iter().map(|(k, _)| *k).collect();
            prop_assert_eq!(tree_keys, sorted);
        }

        #[test]
        fn delete_and_insert_roundtrip(
            keys in prop::collection::vec(0i32..1000, 1..50)
        ) {
            let mut tree = BTree::new();
            for key in &keys {
                tree.insert(*key, *key);
            }

            for key in &keys {
                tree.remove(key);
            }

            prop_assert_eq!(tree.len(), 0);
        }
    }
}
```

```rust
// In tests/integration_btree.rs

use my_db::index::BTree;
use my_db::storage::Storage;

#[test]
fn btree_with_file_storage() {
    let dir = tempfile::tempdir().unwrap();
    let mut storage = Storage::new_file(dir.path()).unwrap();

    // Create tree
    let mut tree = BTree::new();
    for i in 0..100 {
        tree.insert(i, format!("value_{}", i));
    }

    // Simulate persistence (serialize/deserialize)
    let serialized = bincode::serialize(&tree).unwrap();
    let deserialized: BTree<i32, String> = bincode::deserialize(&serialized).unwrap();

    // Verify
    assert_eq!(deserialized.len(), 100);
    assert_eq!(deserialized.get(&50), Some(&"value_50".to_string()));
}
```

---

## Concrete Example: Phase 1 Implementation Plan

Build in this order (each phase is testable and demonstrates concepts):

### Phase 1: Core Storage Layer (Week 1)
```rust
// Deliverables:
// - Page abstraction
// - FileStorage reading/writing fixed-size pages
// - Simple in-memory test backend
// - Test: Write page, read back, verify content

mod storage {
    pub struct Page { data: Vec<u8> }
    pub trait StorageBackend { fn read_page(&self, id) -> Result<Page>; }
    pub struct FileStorage { /* ... */ }
}

// Example: tests/integration_storage.rs
#[test]
fn write_and_read_page() {
    let storage = FileStorage::new("/tmp/test").unwrap();
    let page = Page::new();
    storage.write_page(0, &page).unwrap();
    let read_back = storage.read_page(0).unwrap();
    assert_eq!(page.data(), read_back.data());
}
```

### Phase 2: B-Tree (Week 2)
```rust
// Deliverables:
// - Node structure
// - Insert algorithm
// - Get/Search
// - Validate invariants

mod index {
    pub struct BTree<K, V> { root: Option<Box<Node<K, V>>> }
    pub struct Node<K, V> { keys: Vec<K>, values: Vec<V>, children: Vec<Box<Node>> }
    
    impl<K: Ord, V> BTree<K, V> {
        pub fn insert(&mut self, key: K, value: V) -> Option<V> { /* ... */ }
        pub fn get(&self, key: &K) -> Option<&V> { /* ... */ }
        pub fn validate(&self) -> Result<()> { /* Check invariants */ }
    }
}

// Example: tests/integration_btree.rs
#[test]
fn btree_range_query() {
    let mut tree = BTree::new();
    for i in 0..100 { tree.insert(i, i); }
    let range: Vec<_> = tree.range(20..30).collect();
    assert_eq!(range.len(), 10);
}
```

### Phase 3: Simple Query Layer (Week 3)
```rust
// Deliverables:
// - Simple parser (GET, SET, DELETE, RANGE)
// - Executor connecting to storage + index
// - Example queries

mod query {
    pub enum Query { Get(String), Set(String, Vec<u8>), Range(...) }
    pub struct Executor { /* ... */ }
}

// Example: examples/basic_usage.rs
fn main() {
    let engine = Engine::new("./data").unwrap();
    engine.execute_query("GET key1").unwrap();
    engine.execute_query("SET key1 value1").unwrap();
}
```

---

## Documentation Priorities

For an educational project, prioritize:

1. **Module README comments** (in mod.rs)
   ```rust
   //! # Storage Module
   //!
   //! Handles disk I/O and page management.
   //!
   //! The storage module abstracts file I/O through the `StorageBackend` trait,
   //! allowing both file-based and in-memory implementations for testing.
   ```

2. **Function-level doc comments with examples**
   ```rust
   /// Inserts a key-value pair into the tree.
   ///
   /// # Returns
   /// - `Some(old_value)` if the key already existed
   /// - `None` if this is a new key
   ///
   /// # Example
   /// ```
   /// let mut tree = BTree::new();
   /// let old = tree.insert(1, "first");
   /// assert_eq!(old, None);
   /// ```
   pub fn insert(&mut self, key: K, value: V) -> Option<V> { /* ... */ }
   ```

3. **Architecture document** (`docs/architecture.md`)
   - Why each layer exists
   - How layers communicate
   - Key design decisions and trade-offs

4. **Code examples** (in `examples/`)
   - Basic usage
   - Range queries
   - Persistence workflow

---

## Summary Checklist for Implementation

- [ ] Create `Cargo.toml` with minimal dependencies (just `thiserror`)
- [ ] Set up module hierarchy (error, storage, index, query, db)
- [ ] Implement StorageBackend trait and FileStorage
- [ ] Implement BTree and Node structures
- [ ] Add unit tests in `#[cfg(test)]` blocks
- [ ] Add proptest property tests for B-tree invariants
- [ ] Create integration tests in `tests/`
- [ ] Write doc comments with examples
- [ ] Create 2-3 example files in `examples/`
- [ ] Write README with quick start
- [ ] Document architecture decisions in `docs/architecture.md`

This structure provides:
- **Clear separation of concerns** (each module has one job)
- **Testability** (traits, test doubles, property tests)
- **Extensibility** (traits for storage backends, query executors)
- **Educational value** (code is readable, well-commented, demonstrates patterns)
- **Production-ready patterns** (error handling, validation, but not over-engineered)
