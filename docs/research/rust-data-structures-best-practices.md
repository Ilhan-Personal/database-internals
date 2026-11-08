# Rust Data Structures: Best Practices & Patterns

**Date:** 2026-05-16  
**Context:** Educational codebase for database internals learning project  
**Focus:** Clarity over production performance

---

## 1. Library Structure Patterns

### Crate Organization

#### Recommended layout for a multi-module data structure crate:

```
my_db/
├── Cargo.toml                 # Workspace root or single crate
├── src/
│   ├── lib.rs                 # Crate root; re-exports public API
│   ├── storage/               # Storage layer module
│   │   ├── mod.rs             # Module definition & public API
│   │   ├── page.rs            # Page-based storage abstraction
│   │   ├── file.rs            # File I/O operations
│   │   └── format.rs          # Serialization format helpers
│   ├── index/                 # Indexing layer
│   │   ├── mod.rs
│   │   ├── btree.rs           # B-tree implementation
│   │   ├── node.rs            # Node structures and operations
│   │   └── search.rs          # Search algorithms
│   ├── query/                 # Query parsing and execution
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   └── executor.rs
│   ├── buffer/                # In-memory buffer management
│   │   ├── mod.rs
│   │   └── cache.rs
│   └── error.rs               # Error types (crate-level)
├── examples/                  # Standalone examples
│   ├── basic_usage.rs
│   ├── btree_demo.rs
│   └── persistence.rs
├── benches/                   # Benchmarks (criterion)
│   └── btree_bench.rs
└── tests/                     # Integration tests
    ├── storage_integration.rs
    └── query_integration.rs
```

### Key organizational principles:

**Module root pattern (mod.rs):**
- Each module directory has a `mod.rs` that defines the module boundary
- `mod.rs` re-exports the public API and hides implementation details
- Example: `src/storage/mod.rs` defines what consumers see from the storage module

```rust
// src/storage/mod.rs
pub use self::page::Page;
pub use self::file::FileStorage;
pub use self::format::SerializationFormat;

mod page;
mod file;
mod format;

pub struct Storage { /* ... */ }
pub trait StorageBackend { /* ... */ }
```

**Trait-based extensibility:**
- Define traits at module level for behavior you want to extend
- Allow multiple implementations in submodules
- Facilitates testing with mock implementations

```rust
// src/storage/mod.rs
pub trait StorageBackend: Send + Sync {
    fn read_page(&self, id: PageId) -> io::Result<Page>;
    fn write_page(&mut self, id: PageId, page: &Page) -> io::Result<()>;
}

// src/storage/file.rs
pub struct FileStorage { /* ... */ }
impl StorageBackend for FileStorage { /* ... */ }

// src/storage/memory.rs - for testing
#[cfg(test)]
pub struct MemoryStorage { /* ... */ }
#[cfg(test)]
impl StorageBackend for MemoryStorage { /* ... */ }
```

### crate vs module distinctions:

- **Single crate for monolithic database library** (recommended for educational purposes)
- **Multiple crates in workspace** only if:
  - Crates have independent versioning needs
  - Clear dependency boundaries exist
  - Crates are independently useful
  - For this project: likely overkill; single crate with well-organized modules is cleaner

---

## 2. Testing Patterns for Data Structures

### Test organization:

```
src/
├── index/
│   ├── btree.rs
│   └── btree_tests.rs        # or #[cfg(test)] module in btree.rs
├── storage/
│   ├── file.rs
│   └── tests/                # Optional: for complex storage tests
│       └── file_tests.rs
tests/                        # Integration tests only
├── btree_integration.rs      # Cross-module testing
├── end_to_end.rs            # Full system tests
```

### Unit test patterns (in source files):

```rust
// src/index/btree.rs

pub struct BTree<K, V> { /* ... */ }

impl<K: Ord, V> BTree<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V> { /* ... */ }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_single_key() {
        let mut tree = BTree::new();
        tree.insert(1, "a");
        assert_eq!(tree.get(&1), Some(&"a"));
    }

    #[test]
    fn test_insert_maintains_order() {
        let mut tree = BTree::new();
        for i in (0..100).rev() {
            tree.insert(i, i);
        }
        let mut iter = tree.iter();
        for i in 0..100 {
            assert_eq!(iter.next(), Some((&i, &i)));
        }
    }

    #[test]
    fn test_range_query() {
        let mut tree = BTree::new();
        for i in 0..100 {
            tree.insert(i, i * 2);
        }
        let results: Vec<_> = tree.range(20..30).collect();
        assert_eq!(results.len(), 10);
    }
}
```

### Property-based testing (with proptest):

Best for data structures; catches edge cases and invariants.

```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use super::*;

    proptest! {
        #[test]
        fn btree_maintains_order(mut keys in prop::collection::vec(1..1000i32, 0..100)) {
            keys.sort();
            keys.dedup();
            
            let mut tree = BTree::new();
            for key in &keys {
                tree.insert(*key, *key);
            }

            let retrieved: Vec<_> = tree.iter().map(|(k, _)| *k).collect();
            prop_assert_eq!(retrieved, keys);
        }

        #[test]
        fn insert_delete_invariants(
            ops in prop::collection::vec(
                (any::<i32>(), prop_oneof![Just(Op::Insert), Just(Op::Delete)]),
                0..200
            )
        ) {
            let mut tree = BTree::new();
            let mut expected = std::collections::BTreeMap::new();

            for (key, op) in ops {
                match op {
                    Op::Insert => { tree.insert(key, key); expected.insert(key, key); }
                    Op::Delete => { tree.remove(&key); expected.remove(&key); }
                }
            }

            let tree_keys: Vec<_> = tree.iter().map(|(k, _)| k).collect();
            let expected_keys: Vec<_> = expected.keys().collect();
            prop_assert_eq!(tree_keys, expected_keys);
        }
    }
}
```

### Integration tests (separate test files):

```rust
// tests/btree_integration.rs

use my_db::index::BTree;
use my_db::storage::FileStorage;

#[test]
fn full_flow_with_persistence() {
    let mut tree = BTree::new();
    let storage = FileStorage::new("/tmp/test_btree").unwrap();

    // Insert test data
    for i in 0..1000 {
        tree.insert(i, format!("value_{}", i));
    }

    // Persist
    let serialized = tree.serialize().unwrap();
    storage.write_page(0, &serialized).unwrap();

    // Load and verify
    let data = storage.read_page(0).unwrap();
    let loaded_tree = BTree::deserialize(&data).unwrap();
    
    assert_eq!(loaded_tree.get(&500), tree.get(&500));
}
```

### Testing utilities pattern:

```rust
// src/test_utils.rs (or src/lib.rs under #[cfg(test)])

#[cfg(test)]
pub mod test_utils {
    use crate::index::BTree;

    /// Fixture: Create a pre-populated tree for tests
    pub fn sample_btree() -> BTree<i32, String> {
        let mut tree = BTree::new();
        for i in 0..100 {
            tree.insert(i, format!("value_{}", i));
        }
        tree
    }

    /// Helper: Check tree invariants
    pub fn assert_tree_valid<K: Ord, V>(tree: &BTree<K, V>) {
        // Check height balance, key ordering, node sizes, etc.
        tree.validate().expect("Tree invariants violated");
    }
}
```

### Test organization best practice:

- **Unit tests**: In same file as implementation, `#[cfg(test)] mod tests`
- **Integration tests**: In `tests/` directory, test cross-module interaction
- **Property tests**: In `src/` file with `proptest` crate dependency
- **Benchmarks**: In `benches/` with `criterion` crate
- **Test fixtures/utils**: In separate `test_utils.rs` module

---

## 3. Memory & Concurrency Safety

### For single-threaded educational codebase (initially):

```rust
// Simple interior mutability if needed
use std::cell::RefCell;

pub struct Buffer {
    data: RefCell<Vec<u8>>,
}

impl Buffer {
    pub fn read(&self, offset: usize, len: usize) -> Vec<u8> {
        let data = self.data.borrow();
        data[offset..offset+len].to_vec()
    }
    
    pub fn write(&self, offset: usize, data: &[u8]) {
        let mut buffer = self.data.borrow_mut();
        buffer[offset..offset+data.len()].copy_from_slice(data);
    }
}
```

### Patterns for thread-safe structures (future expansion):

```rust
use std::sync::{Arc, RwLock};
use parking_lot::Mutex;  // Preferred: faster, smaller, fairer than std::sync::Mutex

// Pattern 1: Arc<RwLock<T>> for read-heavy structures
type SharedBTree<K, V> = Arc<RwLock<BTree<K, V>>>;

// Pattern 2: Arc<Mutex<T>> for write-heavy structures
type SharedBuffer = Arc<Mutex<Buffer>>;

// Example usage (when multi-threading is added):
let tree = Arc::new(RwLock::new(BTree::new()));
let tree_clone = Arc::clone(&tree);

std::thread::spawn(move || {
    let mut tree = tree_clone.write();  // Exclusive access
    tree.insert(key, value);
});

// Reader threads
for _ in 0..10 {
    let tree_clone = Arc::clone(&tree);
    std::thread::spawn(move || {
        let tree = tree_clone.read();  // Shared access
        let _ = tree.get(&key);
    });
}
```

### Unsafe boundaries (for B-tree pointer arithmetic):

Keep unsafe code isolated and minimal:

```rust
// src/index/node.rs

pub struct Node<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
    children: Vec<Option<Box<Node<K, V>>>>,  // Prefer Box for owned pointers first
}

impl<K, V> Node<K, V> {
    /// Safe wrapper around pointer arithmetic
    pub fn child_at(&self, index: usize) -> Option<&Node<K, V>> {
        self.children.get(index).and_then(|c| c.as_ref())
    }

    pub fn child_at_mut(&mut self, index: usize) -> Option<&mut Node<K, V>> {
        self.children.get_mut(index).and_then(|c| c.as_mut())
    }

    /// Only use unsafe when absolutely necessary (e.g., custom allocators)
    /// Document why unsafe is needed
    #[allow(unsafe_code)]
    unsafe fn raw_access(&self, index: usize) -> *const V {
        // SAFETY: We ensure index is valid before dereferencing
        if index < self.values.len() {
            self.values.as_ptr().add(index)
        } else {
            std::ptr::null()
        }
    }
}
```

### Memory layout considerations:

```rust
// Prefer field ordering that minimizes padding for performance
pub struct BTreeNode<K: Ord, V> {
    level: u32,           // 4 bytes, aligns with next pointer-sized field
    is_leaf: bool,        // 1 byte → padding needed anyway
    keys: Vec<K>,         // 24 bytes (pointer, len, capacity)
    values: Vec<V>,       // 24 bytes
    children: Vec<Box<Node<K, V>>>,  // 24 bytes
}

// Better organization (larger fields first, padding-aware):
pub struct BTreeNode<K: Ord, V> {
    keys: Vec<K>,         // 24 bytes
    values: Vec<V>,       // 24 bytes
    children: Vec<Box<Node<K, V>>>,  // 24 bytes
    level: u32,           // 4 bytes
    is_leaf: bool,        // 1 byte (compiler pads to 8 bytes anyway)
}
```

---

## 4. Documentation & Examples

### Documentation standards (docs.rs conventions):

```rust
//! # My Database Engine
//!
//! A simple key-value storage engine with B-tree indexing.
//!
//! ## Overview
//!
//! This crate provides a basic storage engine that persists key-value pairs to disk
//! and indexes them for efficient range queries.
//!
//! ## Example
//!
//! ```
//! use my_db::Storage;
//!
//! let mut storage = Storage::new("./data").unwrap();
//! storage.set("key1", b"value1").unwrap();
//! assert_eq!(storage.get("key1").unwrap(), Some(b"value1".to_vec()));
//! ```

pub struct Storage { /* ... */ }

impl Storage {
    /// Creates a new storage engine at the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory where data files will be stored
    ///
    /// # Returns
    ///
    /// Returns a new `Storage` instance, or an error if the directory cannot be created.
    ///
    /// # Example
    ///
    /// ```
    /// use my_db::Storage;
    ///
    /// let storage = Storage::new("./test_data").unwrap();
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        // ...
    }

    /// Retrieves a value by key.
    ///
    /// Returns `Some(value)` if the key exists, `None` otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying I/O operation fails.
    pub fn get(&self, key: &str) -> io::Result<Option<Vec<u8>>> {
        // ...
    }
}
```

### Doc tests (inline executable tests):

```rust
/// Inserts a key-value pair, returning the previous value if present.
///
/// # Example
///
/// ```
/// use my_db::Storage;
///
/// let mut storage = Storage::new("/tmp/test").unwrap();
/// let prev = storage.set("key", b"value").unwrap();
/// assert_eq!(prev, None);
///
/// let prev = storage.set("key", b"new_value").unwrap();
/// assert_eq!(prev, Some(b"value".to_vec()));
/// ```
pub fn set(&mut self, key: &str, value: &[u8]) -> io::Result<Option<Vec<u8>>> {
    // ...
}
```

### Example files (in `/examples/`):

```rust
// examples/basic_usage.rs
//! Demonstrates basic key-value operations

use my_db::Storage;

fn main() -> std::io::Result<()> {
    // Create storage
    let mut storage = Storage::new("./data")?;

    // Set some values
    storage.set("alice", b"alice's value")?;
    storage.set("bob", b"bob's value")?;

    // Retrieve
    if let Some(value) = storage.get("alice")? {
        println!("alice: {}", String::from_utf8_lossy(&value));
    }

    Ok(())
}
```

```rust
// examples/btree_operations.rs
//! Demonstrates B-tree range queries

use my_db::index::BTree;

fn main() {
    let mut tree = BTree::new();

    // Insert ordered data
    for i in 0..100 {
        tree.insert(i, format!("value_{}", i));
    }

    // Range query
    let results: Vec<_> = tree.range(20..30).collect();
    println!("Keys 20-30: {:?}", results);

    // Iteration
    for (key, value) in tree.iter() {
        println!("{}: {}", key, value);
    }
}
```

### README structure:

```markdown
# My Database Engine

A Rust implementation of a simple key-value storage engine with B-tree indexing.

## Features

- Key-value storage with disk persistence
- B-tree indexing for efficient range queries
- Educational focus: clear, well-documented code

## Quick Start

```rust
use my_db::Storage;

let mut storage = Storage::new("./data")?;
storage.set("key", b"value")?;
```

## Examples

See the `examples/` directory for more usage patterns:
- `basic_usage.rs` - Simple key-value operations
- `btree_operations.rs` - Range queries and iteration
- `persistence.rs` - Saving and loading data

## Documentation

Run `cargo doc --open` to view full API documentation.

## Testing

```bash
cargo test                    # Unit and integration tests
cargo test -- --nocapture   # With output
cargo test --doc            # Doc tests only
```

## Benchmarks

```bash
cargo bench
```

## Architecture

See `docs/architecture.md` for design decisions and module breakdown.
```

---

## 5. Dependencies Ecosystem

### Recommended crates for educational database project:

#### Testing:

| Crate | Purpose | Notes |
|-------|---------|-------|
| `proptest` | Property-based testing | Essential for data structures; catches invariant violations |
| `quickcheck` | Property testing (simpler) | Alternative to proptest; more lightweight |
| `criterion` | Benchmarking | Standard for performance testing in Rust |
| `tempfile` | Temporary files in tests | Cleaner test fixtures for file I/O tests |

```toml
[dev-dependencies]
proptest = "1.4"
criterion = { version = "0.5", features = ["html_reports"] }
tempfile = "3.8"
```

#### Concurrency (for future multi-threading):

| Crate | Purpose | Notes |
|-------|---------|-------|
| `parking_lot` | Faster Mutex/RwLock | Drop-in replacement for std::sync; smaller, fairer |
| `crossbeam` | Thread utilities | Channels, work-stealing, scoped threads |
| `dashmap` | Concurrent HashMap | If you add concurrent data structures |
| `rayon` | Data parallelism | For parallel tree operations (future) |

```toml
[dependencies]
parking_lot = "0.12"  # Conditional: only when multi-threading is added

[dev-dependencies]
crossbeam = "0.8"  # For testing concurrent scenarios
```

#### Serialization:

| Crate | Purpose | Notes |
|-------|---------|-------|
| `serde` | Serialization framework | Don't add unless you need flexible formats |
| `bincode` | Binary serialization | Simple, compact; works well with serde |
| `postcard` | Minimal binary format | Lightweight alternative to bincode |

```toml
# Only if serialization needed:
[dependencies]
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
```

#### Utilities:

| Crate | Purpose | Notes |
|-------|---------|-------|
| `anyhow` or `thiserror` | Error handling | `thiserror` better for libraries; `anyhow` for apps |
| `log` | Logging | Standard, minimal overhead |
| `clap` | CLI arguments | For example programs only |

```toml
[dependencies]
thiserror = "1.0"

[dev-dependencies]
log = "0.4"
env_logger = "0.11"
```

### What to avoid for an educational codebase:

- ❌ `tokio` / async runtimes — Keep single-threaded initially
- ❌ Heavy ORM dependencies — Build storage layer yourself
- ❌ Complex macro frameworks — Stick to simple derive macros
- ❌ Database-specific crates (e.g., `rusqlite`) — Build from scratch is the point
- ❌ Multiple testing frameworks — Stick with built-in `#[test]` and proptest

### Minimal starter Cargo.toml:

```toml
[package]
name = "my_db"
version = "0.1.0"
edition = "2021"

[dependencies]
thiserror = "1.0"

[dev-dependencies]
proptest = "1.4"
criterion = { version = "0.5", features = ["html_reports"] }
tempfile = "3.8"

[[example]]
name = "basic_usage"

[[example]]
name = "btree_operations"

[[bench]]
name = "btree_bench"
harness = false
```

---

## 6. Summary: Recommended Patterns for Educational Codebase

### File Structure:
- **Single crate** with modules organized by layer (storage, index, query, buffer)
- **Module boundaries** via `mod.rs` with clear re-exports
- **Traits** for extensibility (storage backends, query executors)
- **Separate test/example modules** for clarity

### Testing:
- **Unit tests** inline with code in `#[cfg(test)] mod tests` blocks
- **Property tests** using `proptest` for invariant checking
- **Integration tests** in `tests/` directory
- **Doc tests** for public API examples
- **Test utilities** in shared `test_utils` module

### Safety & Concurrency:
- **Start single-threaded** with `RefCell` for interior mutability if needed
- **Use `Box<T>` for owned pointers** in tree structures (prefer over raw pointers)
- **Keep `unsafe` isolated** with clear comments on why it's needed
- **Plan for `Arc<RwLock<T>>` / `parking_lot::Mutex`** when multi-threading is added later

### Documentation:
- **Crate-level doc** in `//!` comments in `lib.rs`
- **Module docs** in `mod.rs` files
- **Function docs** with examples that can run as tests
- **Example files** in `examples/` directory
- **README** with quick start and pointer to `cargo doc --open`

### Dependencies:
- **Minimal initially**: Just `thiserror`
- **Testing**: `proptest`, `criterion`, `tempfile`
- **Add `parking_lot`** only when multi-threading is implemented
- **Avoid async/tokio, heavy ORMs, or production database crates**

### Education-First Choices:
- Prioritize **readable code** over micro-optimizations
- Use **clear naming** and module boundaries
- Write **defensive assertions** for invariants
- Document **design decisions** and alternatives considered
- Include **commented examples** showing common patterns

---

## References & Resources

### Key Rust Data Structure Crates to Study:
- `std::collections::BTreeMap` — Rust's standard B-tree implementation
- `parking_lot` — High-performance synchronization primitives
- `crossbeam` — Thread utilities and data structures
- `dashmap` — Concurrent HashMap implementation

### Documentation Tools:
- `cargo doc --open` — Local documentation generation
- https://docs.rs — Published crate documentation
- `rustdoc` — Official documentation format

### Learning Resources:
- *Programming Rust* (Blandy & Orendorff) — Chapter on ownership, traits
- *The Rustonomicon* — Unsafe code and memory layout
- *Rust Design Patterns* (https://rust-lang.github.io/api-guidelines/) — API design

### Example Repositories to Study:
- `std::collections` in rustc source — Production B-tree implementation
- `sled` (key-value store) — Real-world educational project
- `xi-editor` — Complex data structures in Rust
