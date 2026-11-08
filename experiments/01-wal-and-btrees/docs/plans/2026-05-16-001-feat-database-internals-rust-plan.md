---
title: feat: Database Internals Chapter 6 - Rust Implementation
type: feat
status: active
date: 2026-05-16
---

# Database Internals Chapter 6 - Rust Implementation

## Summary

Build an educational Rust library implementing three interconnected database optimization concepts from Database Internals Chapter 6: Write Ahead Logging (WAL) for durability, Copy-on-Write B-Trees for concurrent reads without locking, and Lazy/Lazy Adaptive Trees for buffer-based bulk write optimization. Each concept will be implemented with working examples, integration tests, and clear documentation demonstrating the tradeoffs and mechanisms.

---

## Problem Frame

Understanding how modern databases achieve both durability and performance requires hands-on implementation of their core mechanisms. While reading about WAL, Copy-on-Write, and Lazy Trees provides conceptual knowledge, implementing them in Rust forces you to understand:

- How WAL enables fast writes while maintaining crash-safety
- Why Copy-on-Write eliminates read-write contention
- How buffering trades latency for throughput
- The real performance/durability tradeoffs in each approach

This plan produces a modular, well-tested codebase you can study, modify, and extend as you deepen your database internals knowledge.

---

## Requirements

- R1. Implement a functional Write Ahead Logging (WAL) system that enables crash recovery
- R2. Implement a standard B-tree supporting insert, search, and range queries
- R3. Implement Copy-on-Write B-tree variant allowing concurrent readers and writers
- R4. Implement Lazy Tree with write buffering for bulk operations
- R5. Provide working examples for each concept demonstrating use and behavior
- R6. Include comprehensive tests covering happy path, edge cases, and crash recovery
- R7. Document code with examples showing how each mechanism improves on the previous

---

## Scope Boundaries

- **Not included**: Production optimization (memory allocation patterns, cache-aware indexing), distributed systems (replication, sharding), query optimization or SQL parsing
- **Not included**: Persistence to disk (file I/O abstraction exists, but we use in-memory storage for simplicity)
- **Not included**: Concurrent transactions or MVCC (CoW provides multi-version visibility, but not full ACID transaction support)
- **Educational focus**: Code prioritizes clarity and readability over performance; comments explain *why* decisions exist

---

## Context & Research

### Relevant Patterns

- Rust trait-based extensibility: `StorageBackend` trait for pluggable storage implementations
- Arc/RwLock for thread-safe concurrent access to B-tree structures
- Property-based testing with `proptest` for invariant verification
- Integration tests demonstrating multi-step scenarios (insert, crash, recover)

### External References

- *Database Internals* by Alex Petrov, Chapter 6 (primary source for concepts)
- Rust standard library documentation on Arc, RwLock, and concurrent patterns
- `proptest` documentation for property-based testing strategies

---

## Key Technical Decisions

- **Single in-memory crate**: All three concepts live in one crate (`database_internals`) with separate modules for clarity and composability
- **Arc/RwLock threading model**: Use Rust's standard library Arc/RwLock initially; `parking_lot` for optimized locks in performance-critical sections if needed later
- **No unsafe code initially**: Rely on Rust's ownership system to prevent memory errors; defer unsafe pointer arithmetic for B-tree node traversal if absolutely needed
- **Property-based testing first**: Use `proptest` to generate random operation sequences, verifying invariants hold across all concepts
- **Examples over benchmarks**: Provide runnable `/examples/` programs demonstrating each concept in action, not performance measurements

---

## Output Structure

```
database_internals/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Public module exports
│   ├── error.rs                  # Error types (thiserror)
│   ├── wal/
│   │   ├── mod.rs                # WAL public interface
│   │   ├── log.rs                # Write Ahead Log implementation
│   │   ├── recovery.rs           # Crash recovery logic
│   │   └── tests.rs              # WAL unit tests
│   ├── btree/
│   │   ├── mod.rs                # B-tree public interface & traits
│   │   ├── standard.rs           # Standard B-tree implementation
│   │   ├── copy_on_write.rs      # Copy-on-Write B-tree variant
│   │   ├── lazy.rs               # Lazy tree with buffering
│   │   └── tests.rs              # B-tree unit & property tests
│   ├── storage/
│   │   ├── mod.rs                # Storage trait and in-memory impl
│   │   └── memory.rs             # In-memory storage backend
│   └── buffer/
│       ├── mod.rs                # Buffer management for lazy trees
│       ├── write_buffer.rs       # Write buffer implementation
│       └── tests.rs              # Buffer tests
├── tests/
│   ├── integration_wal.rs        # WAL crash recovery scenarios
│   ├── integration_btree.rs      # Cross-concept integration tests
│   ├── integration_concurrent.rs # Multi-thread scenarios
│   └── property_tests.rs         # Property-based invariant checks
├── examples/
│   ├── wal_basic.rs              # Insert data, crash, recover
│   ├── btree_operations.rs       # Standard B-tree in action
│   ├── cow_concurrent.rs         # CoW B-tree with readers/writers
│   ├── lazy_tree_bulk.rs         # Lazy tree bulk load performance
│   └── full_scenario.rs          # All three concepts working together
└── docs/
    └── CONCEPTS.md               # Explanation of each mechanism
```

---

## High-Level Technical Design

> *This illustrates the architecture and is directional guidance for review. The implementing agent should treat it as context, not implementation specification.*

**Layered Architecture:**

```
┌─────────────────────────────────────────────┐
│  Examples & Integration Tests               │ (How concepts work together)
├─────────────────────────────────────────────┤
│  Lazy Tree | CoW B-Tree | Standard B-Tree   │ (Concept implementations)
├─────────────────────────────────────────────┤
│  WAL System                                 │ (Durability foundation)
├─────────────────────────────────────────────┤
│  Storage Trait + In-Memory Backend          │ (Persistence abstraction)
└─────────────────────────────────────────────┘
```

**Trait-based design allows substitution:**

```rust
// Storage backend trait — in-memory for learning, file-backed in production
pub trait StorageBackend: Send + Sync {
    fn write(&self, key: &[u8], value: &[u8]) -> Result<()>;
    fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn delete(&self, key: &[u8]) -> Result<()>;
}

// B-tree variants all implement same operations trait
pub trait BTree<K, V>: Send + Sync {
    fn insert(&mut self, key: K, value: V) -> Result<()>;
    fn search(&self, key: &K) -> Result<Option<V>>;
    fn range(&self, start: &K, end: &K) -> Result<Vec<(K, V)>>;
}

// WAL wraps any BTree for durability
pub struct WriterAheadLoggedBTree<B: BTree> {
    btree: Arc<RwLock<B>>,
    wal: WriteAheadLog,
}
```

**Data flow for each concept:**

*Standard B-Tree:* Insert/Search/Range operations on in-memory tree directly

*CoW B-Tree:* On write, create new page versions; readers see consistent snapshots via versioned references (Arc/RwLock)

*Lazy Tree:* Writes accumulate in buffer; when buffer fills or on explicit flush, batch-apply to underlying tree

*WAL:* All operations logged before executing; on crash, replay log to recover tree state

---

## Implementation Units

### U1. Project Setup & Storage Layer Foundation

**Goal:** Establish crate structure, error handling, and in-memory storage backend that all concepts will build on.

**Requirements:** R1 (foundation), R7 (documentation)

**Dependencies:** None

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/error.rs`
- Create: `src/storage/mod.rs`
- Create: `src/storage/memory.rs`
- Create: `tests/` directory structure
- Test: `src/storage/memory.rs` tests inline

**Approach:**
- Define `StorageBackend` trait with `write`, `read`, `delete` operations
- Implement in-memory storage using `BTreeMap` as backend
- Use `thiserror` for error types (Result, custom error enums)
- Establish module hierarchy and public API exports in `lib.rs`
- Add documentation comments with examples for all public types

**Patterns to follow:**
- Rust crate structure: `src/`, `tests/`, `examples/` directories
- Trait definition patterns: `pub trait StorageBackend: Send + Sync { ... }`
- Error handling with `thiserror`: `#[derive(thiserror::Error)]`

**Test scenarios:**
- Happy path: Write a key-value pair, read it back, verify value matches
- Edge case: Read non-existent key, returns `Ok(None)` not error
- Edge case: Overwrite existing key, returns new value on read
- Integration: Sequential writes and reads across multiple operations

**Verification:**
- All public types and functions have doc comments with examples
- `cargo test --lib storage` passes with 100% coverage of happy path, edge cases, and error conditions
- `cargo doc --open` renders documentation correctly

---

### U2. Write Ahead Logging (WAL) System

**Goal:** Implement WAL enabling crash recovery — operations are logged before execution, so state can be replayed on restart.

**Requirements:** R1, R6, R7

**Dependencies:** U1

**Files:**
- Create: `src/wal/mod.rs`
- Create: `src/wal/log.rs`
- Create: `src/wal/recovery.rs`
- Create: `src/wal/tests.rs`
- Test: `tests/integration_wal.rs`

**Approach:**
- **Log structure**: Each operation (insert, delete) appended to log as serialized entry with metadata (sequence number, timestamp, checksum for crash safety)
- **Flush strategy**: Log entries flushed before operation completes (durability guarantee)
- **Recovery**: Replay log entries from last checkpoint, reconstructing final state
- **Memory-backed**: Use Vec<LogEntry> for simplicity; abstracted so disk I/O can be swapped in later
- **State machine**: Log entry format includes operation type, key, value, timestamp for replay

**Execution note:** Implement WAL standalone first (verify it works in isolation), then integrate with B-tree in U3.

**Technical design:**

```
LogEntry format (directional):
┌─────────────┬──────────┬─────────┬────────┬──────────┐
│ SequenceNum │ OpType   │ Key     │ Value  │ Checksum │
└─────────────┴──────────┴─────────┴────────┴──────────┘

Recovery process:
1. Load log from checkpoint
2. Filter entries since last checkpoint
3. Replay each entry in order (apply operation to storage)
4. Verify final state matches checkpoint
```

**Patterns to follow:**
- Serialize operations using `serde` (or manual serialization for clarity)
- Arc<Mutex<Vec<LogEntry>>> for thread-safe log access
- Iterator-based log replay (functional style)

**Test scenarios:**
- Happy path: Log operation, verify it appears in log
- Happy path: Crash scenario — log 3 operations, recover, verify all 3 replayed
- Edge case: Recover from empty log (no operations logged)
- Edge case: Duplicate replay of same sequence number (idempotent)
- Error path: Corrupted log entry (checksum fails), recovery halts with error
- Integration: Log sequential insert/delete operations, crash mid-sequence, recover to consistent state

**Verification:**
- `cargo test wal::` passes all unit tests
- `cargo test --test integration_wal` passes crash recovery scenarios
- Example `wal_basic.rs` demonstrates insert, artificial crash, recovery

---

### U3. Standard B-Tree Implementation

**Goal:** Implement standard B-tree (insert, search, range query) with in-memory nodes. Foundation for CoW and Lazy variants.

**Requirements:** R2, R6, R7

**Dependencies:** U1

**Files:**
- Create: `src/btree/mod.rs` (traits, public interface)
- Create: `src/btree/standard.rs` (implementation)
- Create: `src/btree/tests.rs`
- Test: `src/btree/tests.rs` with property-based tests

**Approach:**
- **Node structure**: Internal nodes store keys and child pointers; leaf nodes store key-value pairs
- **Invariants**: Node degree (min/max children per node), key ordering, leaf depth consistency
- **Operations**: Insert with node splitting, search with binary search, range query with tree traversal
- **Thread safety**: Arc<RwLock<BTreeNode>> for node references; readers acquire RwLock::read, writers acquire RwLock::write
- **In-memory**: Nodes stored as Rust structs with Vec for keys/values; no disk I/O
- **Property-based tests**: Generate random sequences of inserts/searches, verify tree invariants after each operation (all keys ordered, tree height correct, etc.)

**Execution note:** Start with failing property-based tests that verify B-tree invariants, then implement tree logic to pass them.

**Patterns to follow:**
- Recursive node operations (insert descends tree, splits nodes as needed)
- BTreeNode enum: `Internal { keys, children }` vs `Leaf { keys, values }`
- Mutable references for insertions (tree modified in place)

**Test scenarios:**
- Happy path: Insert 100 random values, search each one, retrieve all present
- Happy path: Range query returns all keys in [start, end] in sorted order
- Edge case: Empty tree search returns None
- Edge case: Insert duplicate key (overwrites or errors, behavior decision)
- Edge case: Search at tree boundaries (min/max keys)
- Property: After any sequence of inserts, all keys are in order
- Property: All leaves are at same depth
- Property: Each internal node has between min and max children

**Verification:**
- `cargo test btree::standard` passes all unit tests
- `cargo test --test property_tests btree` passes 1000+ randomized insert/search sequences
- Tree maintains all invariants across property test iterations

---

### U4. Copy-on-Write B-Tree Variant

**Goal:** Implement CoW B-tree allowing concurrent readers and writers without blocking. Readers see consistent snapshots; writers create new page versions.

**Requirements:** R3, R6, R7

**Dependencies:** U3

**Files:**
- Create: `src/btree/copy_on_write.rs`
- Modify: `src/btree/mod.rs` (add CoW module export)
- Create: `tests/integration_concurrent.rs`

**Approach:**
- **Version semantics**: Each write creates new versions of affected nodes; old versions remain accessible
- **Reader consistency**: Readers pin to a root version; traverse tree using only nodes from that version
- **Arc for sharing**: Node references wrapped in Arc; multiple versions can share unchanged subtrees
- **Reader-writer separation**: Multiple readers can traverse old versions while writer creates new version
- **Snapshots**: RwLock protects root pointer; readers acquire read lock (non-blocking to each other), writer acquires write lock (blocks new readers, allows existing readers to finish)

**Execution note:** Implement with multi-threaded tests from the start — spawn reader threads that continuously search, writer thread that does bulk inserts, verify readers never see inconsistent state.

**Technical design:**

```
Version progression (directional):
Time 0: Root v0 → [Node A, Node B]
          Reader 1 pins to v0

Time 1: Writer inserts, creates Root v1 → [Node A', Node C]
          (Node A' is new; Node B unchanged)
          Reader 1 still uses v0 (sees old state)

Time 2: New Reader 2 pins to v1 (sees new state)
          Both readers proceed without blocking each other
```

**Patterns to follow:**
- `Arc<RwLock<CoWNode>>` for reference-counted versioned nodes
- `Arc<AtomicPtr<CoWNode>>` for root version pointer (lock-free reads)
- Immutable node operations after creation (functional style)

**Test scenarios:**
- Happy path: Insert, search, range query (same as standard B-tree)
- Concurrent: Reader thread searches while writer inserts; reader never sees partial tree
- Concurrent: Multiple reader threads accessing same CoW tree simultaneously; no blocking
- Edge case: Reader thread pins to old version, then new version created; reader still sees old state correctly
- Integration: Stress test with 10 reader threads + 1 writer thread, 10k operations; verify no panics or data corruption

**Verification:**
- `cargo test btree::copy_on_write` passes unit tests
- `cargo test --test integration_concurrent` passes multi-threaded scenarios without deadlock or race conditions
- Example `cow_concurrent.rs` demonstrates readers and writers operating concurrently

---

### U5. Lazy Tree with Write Buffering

**Goal:** Implement Lazy Tree that buffers writes and applies them in batches, improving throughput by reducing individual tree modifications.

**Requirements:** R4, R6, R7

**Dependencies:** U3 (extends standard B-tree)

**Files:**
- Create: `src/buffer/mod.rs`
- Create: `src/buffer/write_buffer.rs`
- Create: `src/buffer/tests.rs`
- Create: `src/btree/lazy.rs`
- Test: `tests/integration_btree.rs` (compare lazy vs standard throughput)

**Approach:**
- **Write buffer**: Accumulate writes (insert/delete operations) in a BTreeMap or Vec; do not modify main tree yet
- **Flush trigger**: When buffer reaches capacity (configurable, e.g., 1000 ops), or explicit flush() call, apply all buffered operations to main tree at once
- **Deferred visibility**: Reads check buffer first (for buffered-but-not-yet-flushed writes), then tree (for flushed writes)
- **Batching benefit**: Instead of 1000 individual tree modifications (each with node splits), combine related operations and apply them together (fewer total splits)
- **Memory tradeoff**: Buffer uses memory proportional to pending operations; benefit is amortized throughput improvement

**Execution note:** Implement buffer separately from tree, then integrate. Test shows lazy tree throughput improvement over standard tree when doing bulk inserts.

**Technical design:**

```
Lazy Tree structure (directional):
┌──────────────────┐
│ Write Buffer     │ (pending inserts/deletes)
├──────────────────┤
│ B-tree (flushed) │ (committed state)
└──────────────────┘

Read: Check buffer → tree (buffer takes precedence)
Write: Append to buffer
Flush: Apply buffer to tree, clear buffer
```

**Patterns to follow:**
- `struct WriteBuffer<K, V> { pending: BTreeMap<K, Operation<V>> }`
- Lazy tree wraps both buffer and B-tree: `struct LazyTree { buffer: WriteBuffer, tree: StandardBTree }`
- `pub fn flush(&mut self)` applies buffered ops to tree

**Test scenarios:**
- Happy path: Insert into buffer, verify not visible until flush
- Happy path: Insert into buffer, search returns buffered value before flush
- Happy path: Insert 1000 items via lazy tree, flush once, compare final tree to one built with 1000 individual inserts (same result)
- Edge case: Buffer capacity boundary (insert exactly capacity, flush, insert over capacity, verify flush triggered)
- Performance: Time insertion of 10k items (lazy tree buffer batch vs standard tree individual inserts)
- Integration: Lazy tree with WAL — inserts buffered then flushed, WAL logs flush operation (not individual buffer ops)

**Verification:**
- `cargo test buffer::` passes buffer unit tests
- `cargo test btree::lazy` passes lazy tree tests
- `cargo test --test integration_btree` passes cross-tree scenarios
- Example `lazy_tree_bulk.rs` demonstrates bulk load with batching

---

### U6. WAL Integration with All B-Tree Variants

**Goal:** Integrate WAL with standard, CoW, and Lazy B-trees, enabling crash recovery for all concepts.

**Requirements:** R1, R5, R6, R7

**Dependencies:** U2, U3, U4, U5

**Files:**
- Create: `src/wal/btree_wrapper.rs`
- Modify: `tests/integration_wal.rs` (add CoW and Lazy tree scenarios)

**Approach:**
- **Wrapper pattern**: `WriterAheadLoggedBTree<B: BTree>` wraps any BTree variant
- **Operation interception**: Before executing insert/delete/range, append operation to WAL log
- **Recovery**: On startup, load stored tree snapshot and replay log entries
- **Flush semantics**: WAL log is flushed to storage before operation completes; tree modification can be async
- **All variants supported**: Wrapper works identically with Standard, CoW, and Lazy variants

**Patterns to follow:**
- Generic wrapper: `impl<B: BTree> WriterAheadLoggedBTree<B> { ... }`
- Delegate methods to inner B-tree after logging
- Recovery function reconstructs tree state by replaying log

**Test scenarios:**
- Happy path: Wrap standard B-tree with WAL, insert/search works
- Happy path: Wrap CoW B-tree with WAL, concurrent reads during recovery
- Happy path: Wrap Lazy B-tree with WAL, buffer flush recorded in log
- Crash: Wrapped standard tree crashes mid-insert, recover via WAL replay
- Crash: Wrapped CoW tree crashes, readers from old version still consistent post-recovery
- Crash: Wrapped Lazy tree crashes, buffer operations reapplied via WAL
- Error path: Log corruption detected during recovery, recovery halts gracefully

**Verification:**
- `cargo test wal::btree_wrapper` passes all variants
- `cargo test --test integration_wal` passes crash recovery for each variant
- No regressions in existing B-tree or WAL tests

---

### U7. Examples and Documentation

**Goal:** Create runnable examples demonstrating each concept in isolation and in combination. Document design decisions and tradeoffs.

**Requirements:** R5, R7

**Dependencies:** U1-U6

**Files:**
- Create: `examples/wal_basic.rs`
- Create: `examples/btree_operations.rs`
- Create: `examples/cow_concurrent.rs`
- Create: `examples/lazy_tree_bulk.rs`
- Create: `examples/full_scenario.rs`
- Create: `docs/CONCEPTS.md`
- Modify: `src/lib.rs` module docs

**Approach:**
- **wal_basic.rs**: Insert 100 items, simulate crash (drop tree), recover via WAL, verify all items restored
- **btree_operations.rs**: Standard B-tree insert, search, range query with visual output
- **cow_concurrent.rs**: Spawn reader threads that continuously search, writer thread that inserts; show no contention
- **lazy_tree_bulk.rs**: Time bulk load with lazy tree vs standard tree, show throughput improvement
- **full_scenario.rs**: Create WAL-backed CoW Lazy tree, run mixed workload, explain each decision
- **CONCEPTS.md**: Explain WAL recovery process, CoW multi-versioning, Lazy batching; link to code

**Patterns to follow:**
- Use `main()` with clear print statements showing what's happening
- Simple data (strings or integers), not domain-specific data
- Include comments explaining "why" for each design choice

**Test scenarios:**
- Examples compile and run without errors
- Output is deterministic and understandable
- Examples demonstrate the key behavior of each concept

**Verification:**
- `cargo run --example wal_basic` completes successfully
- `cargo run --example cow_concurrent` shows concurrent operations happening
- `cargo run --example lazy_tree_bulk` produces timing comparison
- `cargo run --example full_scenario` runs without crashes
- Documentation is comprehensive and links code examples

---

### U8. Property-Based Testing & Test Suite

**Goal:** Comprehensive property-based testing verifying invariants across all concepts and their interactions.

**Requirements:** R6, R7

**Dependencies:** U1-U7

**Files:**
- Create/Modify: `tests/property_tests.rs`
- Create/Modify: `src/btree/tests.rs` (add proptest strategies)

**Approach:**
- **Properties to verify**: 
  - B-tree invariants (ordering, depth consistency) after any insert sequence
  - CoW version consistency (readers see correct state regardless of concurrent writes)
  - Lazy tree correctness (buffered operations produce same result as immediate operations)
  - WAL idempotence (replaying same log twice produces same state)
- **Strategies**: Random insert/delete/search sequences using `proptest::proptest!` macro
- **Shrinking**: When property fails, proptest shrinks failing input to minimal reproducer

**Patterns to follow:**
- Use `proptest::strategy` to generate random operation sequences
- Verify invariants after each operation
- Run with high iteration count (1000+) to increase confidence

**Test scenarios:**
- Property: After any sequence of inserts into B-tree, all keys are in sorted order (checked by tree traversal)
- Property: After inserting N items, search finds all N items
- Property: CoW tree with concurrent reader sees snapshot-consistent state
- Property: Lazy tree with same ops as standard tree produces identical final state
- Property: WAL replay is idempotent (replay twice = replay once)
- Property: Deletes reduce item count correctly

**Verification:**
- `cargo test --test property_tests` runs 1000+ iterations per property
- All properties pass without failures or regressions
- `cargo test --all` passes all unit, integration, and property tests

---

## System-Wide Impact

- **No external dependencies in core library**: Storage trait allows future swapping (e.g., file-backed storage)
- **No unsafe code required**: Rust's ownership system prevents memory errors; thread safety via Arc/RwLock
- **Error propagation**: Result types propagated through all layers (WAL, B-tree, storage)
- **Backward compatibility**: Not applicable (greenfield project)
- **Testing philosophy**: Every concept tested in isolation and in integration; property-based tests catch invariant violations early

---

## Risks & Dependencies

| Risk | Mitigation |
|------|------------|
| CoW tree complexity (versioning, reference counting) hard to understand | Implement with clear variable names, extensive comments, and concrete examples; test with visual debugging |
| WAL recovery logic has edge cases (partial writes, log corruption) | Implement recovery with checksums; unit test all error paths |
| Lazy tree buffer correctness (reads see both buffered and flushed state) | Test with many concurrent reads/writes; property tests verify final state matches immediate writes |
| Performance differences hard to measure (examples use simple data) | Accept as learning limitation; final example shows relative timing but caveats about production |

---

## Documentation / Operational Notes

- **Crate root docs** (`src/lib.rs`): High-level overview of three concepts and how they interact
- **Module docs** (`src/wal/mod.rs`, `src/btree/mod.rs`, etc.): Explain each concept, tradeoffs, use cases
- **Function docs**: All public functions have `///` doc comments with examples
- **Examples**: Run with `cargo run --example <name>` to see each concept in action
- **Concepts guide**: `docs/CONCEPTS.md` provides learning narrative linking code to theory

---

## Sources & References

- *Database Internals* by Alex Petrov, Chapter 6
- Rust documentation: Arc, RwLock, ownership system
- `proptest` documentation for property-based testing
- Research documents: `/Users/ilhan/personal/database_internals/docs/research/`
