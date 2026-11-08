# Quick Start: Rust Data Structures Best Practices

**For your database internals project**

---

## The Core Decisions (TL;DR)

### 1. **Project Structure**
- **Single crate** with modules: `storage/`, `index/`, `query/`, `buffer/`, `error.rs`
- Each module has `mod.rs` that re-exports public API
- Traits define extensibility points (e.g., `StorageBackend`)

### 2. **Testing Strategy**
- **Unit tests**: `#[cfg(test)] mod tests` in same file as implementation
- **Property tests**: `proptest` crate for invariant checking (essential for B-trees)
- **Integration tests**: `tests/` directory for cross-module scenarios
- **Doc tests**: Examples in doc comments that compile and run

### 3. **Safety (Single-threaded for now)**
- Rust's ownership system handles memory safety automatically
- Use `Result<T>` for error handling, not panics
- Add `Arc<RwLock<T>>` and `parking_lot` only when multi-threading needed

### 4. **Documentation**
- Crate docs in `lib.rs` using `//!`
- Module docs in `mod.rs` using `//!`
- Function docs with examples using `///`
- Examples in `examples/` directory
- Inline code comments explaining "why", not "what"

### 5. **Dependencies (Minimal)**
- **Required**: `thiserror` (error handling)
- **Dev**: `proptest`, `criterion`, `tempfile` (testing)
- **Future**: `parking_lot` (when multi-threading), `serde` + `bincode` (if serialization needed)

---

## Concrete Starter Cargo.toml

```toml
[package]
name = "database-engine"
version = "0.1.0"
edition = "2021"

[dependencies]
thiserror = "1.0"

[dev-dependencies]
proptest = "1.4"
criterion = { version = "0.5", features = ["html_reports"] }
tempfile = "3.8"

[[example]]
name = "basic_kvstore"

[[bench]]
name = "btree_operations"
harness = false
```

---

## File Structure You Should Create

```
database-engine/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Crate root: //! module docs
│   ├── error.rs                  # All error types (thiserror)
│   │
│   ├── storage/
│   │   ├── mod.rs                # trait StorageBackend + public API
│   │   ├── page.rs               # struct Page
│   │   ├── file_storage.rs       # impl StorageBackend for FileStorage
│   │   └── format.rs             # Serialization helpers
│   │
│   ├── index/
│   │   ├── mod.rs                # struct BTree + public operations
│   │   ├── btree.rs              # BTree impl + #[cfg(test)] unit tests
│   │   ├── node.rs               # struct Node + node operations
│   │   ├── search.rs             # Search algorithms
│   │   └── iter.rs               # Iterator implementations
│   │
│   ├── query/
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   └── executor.rs
│   │
│   └── db.rs                     # Top-level Engine struct
│
├── examples/
│   ├── basic_kvstore.rs
│   └── btree_demo.rs
│
└── tests/
    ├── integration_storage.rs
    └── integration_btree.rs
```

---

## Copy-Paste Starting Point for Key Files

### lib.rs
```rust
//! # Database Storage Engine
//!
//! A simple key-value storage engine with B-tree indexing.
//!
//! ## Overview
//!
//! This crate provides a basic storage engine that persists key-value pairs
//! to disk and indexes them using B-trees for efficient range queries.
//!
//! ## Example
//!
//! ```
//! use database_engine::Engine;
//!
//! let mut engine = Engine::new("./data").unwrap();
//! engine.set("key1", b"value1").unwrap();
//! assert_eq!(engine.get("key1").unwrap(), Some(b"value1".to_vec()));
//! ```

pub mod error;
pub mod storage;
pub mod index;
pub mod query;
pub mod db;

pub use error::{Result, IndexResult};
pub use db::Engine;
```

### error.rs
```rust
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

    #[error("Invariant violated: {0}")]
    InvariantViolation(String),
}

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("Tree corrupted: {0}")]
    Corrupted(String),

    #[error("Key not found")]
    KeyNotFound,
}

pub type Result<T> = std::result::Result<T, StorageError>;
pub type IndexResult<T> = std::result::Result<T, IndexError>;
```

### storage/mod.rs
```rust
//! Storage layer: Disk I/O and page management
//!
//! The storage module provides the `StorageBackend` trait, which allows
//! different implementations (file-based, in-memory for testing, etc.)

pub use self::page::Page;
pub use self::file_storage::FileStorage;

mod page;
mod file_storage;
mod format;

#[cfg(test)]
mod memory_storage;

pub type PageId = u32;

/// Trait for pluggable storage backends
pub trait StorageBackend: Send + Sync {
    fn read_page(&self, id: PageId) -> crate::Result<Page>;
    fn write_page(&mut self, id: PageId, page: &Page) -> crate::Result<()>;
    fn page_count(&self) -> crate::Result<u32>;
}

/// Main storage engine
pub struct Storage {
    backend: Box<dyn StorageBackend>,
}

impl Storage {
    pub fn new_file<P: AsRef<std::path::Path>>(path: P) -> crate::Result<Self> {
        Ok(Storage {
            backend: Box::new(FileStorage::new(path)?),
        })
    }

    pub fn get_page(&self, id: PageId) -> crate::Result<Page> {
        self.backend.read_page(id)
    }

    pub fn put_page(&mut self, id: PageId, page: &Page) -> crate::Result<()> {
        self.backend.write_page(id, page)
    }
}
```

### index/mod.rs
```rust
//! B-tree index layer
//!
//! Provides the `BTree` data structure for ordered key-value storage
//! with efficient range queries.

pub use self::btree::BTree;

mod btree;
mod node;
mod search;
mod iter;

pub use btree::BTree;
```

### index/btree.rs
```rust
use crate::IndexResult;

/// Ordered key-value storage using a B-tree
pub struct BTree<K, V> {
    root: Option<Box<super::node::Node<K, V>>>,
    len: usize,
    order: usize,
}

impl<K: Ord, V> BTree<K, V> {
    pub fn new() -> Self {
        BTree {
            root: None,
            len: 0,
            order: 4,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // TODO: implement insertion
        None
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        // TODO: implement lookup
        None
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        // TODO: implement deletion
        None
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn validate(&self) -> IndexResult<()> {
        // TODO: validate tree invariants
        Ok(())
    }
}

impl<K: Ord, V> Default for BTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tree() {
        let tree: BTree<i32, i32> = BTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let mut tree = BTree::new();
        tree.insert(10, "a");
        assert_eq!(tree.get(&10), Some(&"a"));
        assert_eq!(tree.get(&20), None);
    }
}
```

---

## Testing: What to Do Immediately

### 1. Add unit tests to EVERY module
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operation() {
        // TODO: add test
    }
}
```

### 2. Add property tests for data structures
```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_insert_maintains_order(keys in prop::collection::vec(0i32..1000, 0..100)) {
            // TODO: implement property test
        }
    }
}
```

### 3. Add integration tests
Create `tests/integration_btree.rs`:
```rust
use database_engine::index::BTree;

#[test]
fn btree_with_many_operations() {
    let mut tree = BTree::new();
    for i in 0..100 {
        tree.insert(i, i);
    }
    assert_eq!(tree.len(), 100);
}
```

### 4. Run tests
```bash
cargo test                    # Run all tests
cargo test -- --nocapture   # See output
cargo test --doc            # Doc tests only
```

---

## Documentation: What to Write

### For every public module:
```rust
//! # Module Name
//!
//! Brief description of what this module does.
//!
//! ## Overview
//! More detailed explanation.
//!
//! ## Example
//! ```
//! // Code that actually compiles and runs
//! ```
```

### For every public function:
```rust
/// What this function does.
///
/// # Arguments
/// * `arg1` - What this argument means
///
/// # Returns
/// Describes the return value or error
///
/// # Example
/// ```
/// // Compilable example
/// ```
pub fn my_function(arg1: i32) -> Result<String> {
    // ...
}
```

### Create examples/basic_kvstore.rs:
```rust
//! Basic key-value store demonstration

use database_engine::Engine;

fn main() -> database_engine::Result<()> {
    let mut engine = Engine::new("./data")?;
    
    engine.set("alice", b"alice's data")?;
    engine.set("bob", b"bob's data")?;
    
    if let Some(data) = engine.get("alice")? {
        println!("alice: {}", String::from_utf8_lossy(&data));
    }
    
    Ok(())
}
```

---

## Key Principles to Remember

1. **Compiler is your friend**: It enforces memory safety. Trust it.
2. **Traits are for extensibility**: Use them for `StorageBackend`, `Query`, etc.
3. **Tests are documentation**: Well-tested code shows how to use it.
4. **Properties catch bugs**: Test invariants with proptest, not just happy paths.
5. **Errors are expected**: Use `Result<T>`, never `unwrap()` in library code.
6. **Keep it simple**: The goal is education, not production optimization.

---

## Checklist: Before You Start Coding

- [ ] Create project structure (directories + mod.rs files)
- [ ] Define error types in error.rs
- [ ] Define StorageBackend trait and Page struct
- [ ] Sketch BTree::insert, BTree::get, BTree::remove (TODO stubs)
- [ ] Add basic unit tests for each
- [ ] Run `cargo test` and see what breaks
- [ ] Write property tests for invariants
- [ ] Write integration tests across modules
- [ ] Document with examples
- [ ] Run `cargo doc --open` and review

---

## Quick Reference: When to Use What

| Need | Solution |
|------|----------|
| Error handling | `thiserror` + Result types |
| Testing data structures | `proptest` for property tests |
| Multiple implementations | Traits (e.g., StorageBackend) |
| Thread-safe in future | `parking_lot::RwLock` + `Arc` (add later) |
| Documentation | Doc comments (`///`) with examples |
| Examples for users | Files in `examples/` |
| Performance testing | `criterion` benchmarks |
| Temporary test files | `tempfile` crate |

---

## Recommended Reading Order

1. **This document** — Overview and decisions
2. **rust-data-structures-best-practices.md** — Deep patterns (dependencies, traits, testing)
3. **architecture-recommendations.md** — Specific to your database engine
4. **testing-safety-patterns.md** — Testing and safety deep dive

Then start coding!
