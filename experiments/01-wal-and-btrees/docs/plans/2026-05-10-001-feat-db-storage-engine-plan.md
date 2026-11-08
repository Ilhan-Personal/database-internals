---
title: Database Storage Engine in Rust
type: feat
status: active
date: 2026-05-10
origin: docs/brainstorms/db-storage-engine-requirements.md
---

# Database Storage Engine in Rust

## Summary

Build a **proof-of-concept database storage engine in Rust** that demonstrates core database concepts through working code: a key-value store with B-tree indexing, disk persistence, and a simple query engine. The implementation focuses on clarity and learning value — each layer (storage, index, query) is designed to be readable code that illustrates how databases actually organize and retrieve data.

---

## Problem Frame

Database internals are often taught at a high level. This project builds from scratch to understand the concrete mechanics: how data structures like B-trees enable efficient retrieval, how disk I/O is organized, and how queries are executed. Starting with a proof-of-concept scope (few sessions) lets us ship something functional while staying focused on clarity.

---

## Requirements

- R1. Store and retrieve key-value pairs using simple operations (get, set, delete)
- R2. Persist data to disk so it survives process restarts
- R3. Support efficient range queries and ordered iteration via B-tree indexing
- R4. Execute simple SQL-like queries against stored data
- R5. Code is readable and well-structured to demonstrate concepts clearly
- R6. Basic test coverage validates core functionality

---

## Scope Boundaries

**Explicit non-goals (for now):**
- Transactions and ACID properties (single-statement operations only)
- Concurrent access or thread-safety (single-threaded)
- Query optimization or cost-based planning (naive execution)
- Advanced indexing beyond B-tree (no hash tables, full-text search, secondary indexes)
- Network protocol or server mode (in-process only)
- High performance or production optimization

### Deferred to Follow-Up Work

- Query planning and optimization
- Concurrent read/write support
- Transaction support and ACID guarantees
- Secondary indexes and advanced query features
- API server / network protocol

---

## Context & Research

### Relevant Patterns

This is a greenfield project. The implementation follows standard Rust conventions:
- Modular structure with clear separation of concerns (storage, index, query layers)
- Error handling via `Result<T, E>` and custom error types
- Serialization via `serde` for on-disk format
- Testing with `#[cfg(test)]` modules

### External References

- **Database Design:** Understanding B-tree structure, page-oriented I/O, and query execution from first principles
- **Rust Serialization:** Using `serde` + `bincode` for efficient binary serialization
- **File I/O:** Rust's `std::fs` for disk operations; memory-mapped I/O considered for future optimization

---

## Key Technical Decisions

- **Binary serialization format:** Use `serde` + `bincode` for efficient, compact on-disk representation. Simple to implement and understand, sufficient for POC.
- **In-memory B-tree for index:** Build the B-tree entirely in memory during the POC phase. This clarifies the data structure first; disk-backed pages can be added later.
- **Simple record storage:** Records stored sequentially with metadata (key length, value length) for easy parsing. No compression or complex encoding in the initial version.
- **Query language:** Simple custom DSL (not SQL) to keep parsing straightforward. Operations: `GET key`, `SET key value`, `DELETE key`, `RANGE start end`, `SCAN`.
- **Error handling:** Use explicit `Result` types with custom error variants for disk I/O, parsing, and key-not-found scenarios.

---

## Open Questions

### Resolved During Planning

- **B-tree node structure:** In-memory nodes with simple Vec for children; no complex pointer management. Trade memory efficiency for clarity.
- **Query response format:** Return results as `Vec<(String, String)>` tuples. Strings for both keys and values in the POC for simplicity.
- **Disk file format:** Single file per database. Start simple; log-structured or multi-file can come later if needed.

### Deferred to Implementation

- **Exact B-tree order:** Will be determined once we prototype; likely 4-8 children per node to balance simplicity and depth.
- **Buffer management details:** Initial in-memory working set sufficient; LRU cache can be added if needed for larger datasets.
- **Error recovery:** Basic error messages sufficient for POC; recovery strategies can be enhanced later.

---

## Output Structure

```
src/
├── main.rs              # Entry point and REPL
├── storage/
│   ├── mod.rs           # Storage layer interface
│   ├── file_store.rs    # File I/O and serialization
│   └── record.rs        # Record format and codecs
├── index/
│   ├── mod.rs           # Index layer interface
│   └── btree.rs         # B-tree implementation
├── query/
│   ├── mod.rs           # Query layer interface
│   ├── parser.rs        # Query DSL parser
│   └── executor.rs      # Query executor
└── db.rs                # Main database engine facade

tests/
├── integration_tests.rs # End-to-end tests
└── ...

Cargo.toml
```

---

## High-Level Technical Design

> *This illustrates the intended approach and is directional guidance for review, not implementation specification. The implementing agent should treat it as context, not code to reproduce.*

### Data Flow

```
┌─────────────────────────────────────────────────────────┐
│  User Query (CLI or API)                                │
└────────────────┬────────────────────────────────────────┘
                 │
                 ▼
         ┌──────────────────┐
         │  Query Parser    │
         │  (parse DSL)     │
         └────────┬─────────┘
                  │
                  ▼
          ┌───────────────────┐
          │  Query Executor   │
          │  (execute ops)    │
          └────────┬──────────┘
                   │
        ┌──────────┼──────────┐
        │          │          │
        ▼          ▼          ▼
     [GET]     [RANGE]    [SCAN]
        │          │          │
        └──────────┼──────────┘
                   │
                   ▼
          ┌───────────────────┐
          │  B-tree Index     │
          │  (lookup key)     │
          └────────┬──────────┘
                   │
                   ▼
        ┌──────────────────────┐
        │  Storage Layer       │
        │  (fetch from disk)   │
        └────────┬─────────────┘
                 │
                 ▼
        ┌──────────────────────┐
        │  File I/O            │
        │  (read records)      │
        └────────┬─────────────┘
                 │
                 ▼
        ┌──────────────────────┐
        │  Results             │
        └──────────────────────┘
```

### Component Interaction

- **Storage Layer:** Manages disk I/O and serialization. Responsible for reading and writing records from/to disk. Exposes `write_record(key, value)` and `read_record(key)` operations.
- **Index Layer (B-tree):** Maintains an in-memory B-tree index of all keys. Maps keys to disk locations. Supports range queries and ordered iteration.
- **Query Layer:** Parses DSL queries and delegates to executor. Executor consults B-tree, retrieves data from storage, and formats results.
- **Database Facade:** Coordinates all layers, manages initialization, and exposes the public API.

---

## Implementation Units

### U1. Project Setup and Core Module Structure

**Goal:** Initialize a Rust project with the modular structure needed for all layers (storage, index, query, main database engine).

**Requirements:** R5 (readable structure)

**Dependencies:** None

**Files:**
- Create: `Cargo.toml` (with dependencies: `serde`, `bincode`)
- Create: `src/main.rs` (entry point)
- Create: `src/db.rs` (database facade)
- Create: `src/storage/mod.rs`, `src/storage/file_store.rs`, `src/storage/record.rs`
- Create: `src/index/mod.rs`, `src/index/btree.rs`
- Create: `src/query/mod.rs`, `src/query/parser.rs`, `src/query/executor.rs`
- Create: `tests/integration_tests.rs`

**Approach:**
- Set up a standard Rust binary project with appropriate module organization
- Define public APIs for each layer (storage, index, query) as traits or structs
- Export from each module so the main database facade can coordinate
- Include `serde` and `bincode` in `Cargo.toml` for serialization

**Patterns to follow:**
- Rust module organization conventions (one module per file, module tree in `mod.rs`)
- Error handling via custom error types in each layer

**Test scenarios:**
- Test expectation: none -- this is scaffolding; functional tests happen in later units

**Verification:**
- Project compiles without errors
- All modules are accessible from the main database facade
- `cargo build` succeeds

---

### U2. Storage Layer: File I/O and Record Format

**Goal:** Implement disk-based storage with a simple record format so data can be written and read back correctly.

**Requirements:** R2 (persistence), R1 (get/set operations)

**Dependencies:** U1

**Files:**
- Modify: `src/storage/mod.rs` — define `StorageEngine` trait
- Modify: `src/storage/file_store.rs` — implement file-based storage
- Modify: `src/storage/record.rs` — define record format (key length, value length, data)
- Create: `tests/storage_tests.rs`

**Approach:**
- Define a `StorageEngine` trait with `write_record(key, value) -> Result<u64>` (returns disk offset) and `read_record(offset) -> Result<(String, String)>`.
- Implement `FileStore` which:
  - Opens/creates a single database file on disk
  - Writes records sequentially with metadata headers (key length as u32, value length as u32, followed by key and value bytes)
  - Reads records by seeking to an offset and parsing the metadata
  - Maintains an in-memory map of current file size to know where to append next
- Use `serde` + `bincode` for serialization of metadata; raw bytes for key/value data
- Use `std::fs::File` for file I/O with `Seek` and `Write`/`Read` traits

**Technical design:**
```
Record Format (on disk):
┌─────────────────────────────────┐
│ key_len: u32 (4 bytes)          │
├─────────────────────────────────┤
│ value_len: u32 (4 bytes)        │
├─────────────────────────────────┤
│ key: [u8; key_len]              │
├─────────────────────────────────┤
│ value: [u8; value_len]          │
└─────────────────────────────────┘
```

**Patterns to follow:**
- Use `Result<T>` for error handling; define custom `StorageError` enum
- Follow Rust file I/O patterns with `seek()`, `read()`, `write()`

**Test scenarios:**
- Happy path: Write a record, read it back by offset, verify key and value match
- Edge case: Write multiple records sequentially, read each by its offset
- Edge case: Write and read empty values
- Error path: Attempt to read from an invalid offset
- Error path: Disk write failure (simulate by insufficient permissions)

**Verification:**
- Records written to disk can be read back exactly as written
- Offsets are calculated correctly to find records on subsequent reads
- File persists after the program exits and can be reopened

---

### U3. Index Layer: B-Tree Implementation

**Goal:** Build an in-memory B-tree that maps keys to disk storage offsets, supporting efficient lookup and range queries.

**Requirements:** R3 (range queries, ordered iteration), R1 (efficient lookup)

**Dependencies:** U1

**Files:**
- Modify: `src/index/mod.rs` — define `Index` trait
- Modify: `src/index/btree.rs` — implement B-tree
- Create: `tests/btree_tests.rs`

**Approach:**
- Define an `Index` trait with methods: `insert(key, offset)`, `search(key) -> Option<u64>`, `range(start, end) -> Vec<(String, u64)>`, `iter() -> Vec<(String, u64)>` (sorted by key).
- Implement a simple B-tree:
  - Use generics over key type; for POC, only support `String` keys
  - Each node stores up to `MAX_KEYS` keys and `MAX_KEYS + 1` children (binary tree-like for simplicity; `MAX_KEYS = 4` to keep it simple)
  - Support insertion with basic splitting when a node exceeds capacity
  - Support search via recursive descent
  - Support range queries via in-order traversal with predicates
  - Keep all nodes in memory (no disk persistence of index itself)

**Technical design:**
```
B-Tree Node Structure (in-memory):
pub struct BTreeNode {
    keys: Vec<String>,           // sorted keys
    children: Vec<Box<BTreeNode>>, // child pointers (None for leaf)
    offsets: Vec<u64>,           // disk offsets for each key
    is_leaf: bool,
}
```

**Patterns to follow:**
- Use `Box<T>` for owned heap allocation of child nodes
- Implement `Ord` and `Eq` for keys to enable sorting and comparison
- Use recursive algorithms for insert, search, and range traversal

**Test scenarios:**
- Happy path: Insert keys in sorted order, search for each, verify offsets returned
- Edge case: Insert keys in random order, verify tree maintains sorted property
- Edge case: Range query with start == end, verify single key returned
- Edge case: Range query with no matching keys
- Happy path: Iterate all keys in order, verify order is maintained
- Edge case: Insert duplicate key, verify behavior (replace or error)

**Verification:**
- B-tree maintains sorted key order after insertions
- Searches return correct offsets for all inserted keys
- Range queries return only keys within bounds, in sorted order
- In-order iteration returns all keys sorted

---

### U4. Storage & Index Integration: Database Facade

**Goal:** Integrate storage and index layers into a unified database engine that coordinates write and read operations.

**Requirements:** R1 (get/set/delete), R2 (persistence), R3 (B-tree integration)

**Dependencies:** U2, U3

**Files:**
- Modify: `src/db.rs` — implement `Database` struct
- Create: `tests/db_integration_tests.rs`

**Approach:**
- Implement a `Database` struct that:
  - Owns both a `FileStore` (storage) and a `BTree` (index)
  - On initialization, loads the existing database file (if it exists) and rebuilds the B-tree index by scanning all records
  - Exposes `set(key, value)` which writes to storage, gets the offset, and updates the index
  - Exposes `get(key)` which queries the index, retrieves the offset, and reads from storage
  - Exposes `delete(key)` which removes from the index (marking in storage is optional for POC)
  - Exposes `range(start, end)` which delegates to the index and retrieves all matching values

**Patterns to follow:**
- Use a single `Database` struct as the main public API
- Coordinate between layers via method calls; keep layers loosely coupled via traits

**Test scenarios:**
- Happy path: Create database, set key-value, get key, verify value matches
- Happy path: Set multiple key-values, retrieve each, verify all correct
- Happy path: Delete a key, verify subsequent get returns not-found error
- Happy path: Query range, verify only matching keys returned in order
- Happy path: Set, then restart (reload from disk), verify data persists
- Edge case: Open a database file created in a previous session, verify index rebuilt correctly
- Error path: Get a key that doesn't exist
- Integration: Set and range query in same session

**Verification:**
- All set/get/delete operations work as expected
- Data persists across restarts
- Range queries return correct sorted results
- Index is correctly rebuilt from disk on startup

---

### U5. Query Layer: Parser and Executor

**Goal:** Implement a simple query DSL parser and executor so users can execute commands like `GET key`, `SET key value`, `RANGE start end`.

**Requirements:** R4 (query execution)

**Dependencies:** U4

**Files:**
- Modify: `src/query/mod.rs` — define query types
- Modify: `src/query/parser.rs` — implement parser
- Modify: `src/query/executor.rs` — implement executor
- Create: `tests/query_tests.rs`

**Approach:**
- Define an enum for supported query operations: `Query::Get(key)`, `Query::Set(key, value)`, `Query::Delete(key)`, `Query::Range(start, end)`, `Query::Scan`.
- Implement a simple parser that:
  - Splits input by whitespace
  - Matches the first token to determine operation
  - Extracts operands
  - Returns a `Query` enum or a parse error
  - Example: `"GET user:1"` → `Query::Get("user:1")`
- Implement an executor that:
  - Takes a `Query` and a `&mut Database`
  - Executes the operation and returns results as formatted strings
  - Example: `Query::Get("user:1")` → calls `database.get("user:1")` → formats result

**Technical design:**
```
Query DSL Examples:
GET key                    -- retrieve value for key
SET key value              -- store key-value pair
DELETE key                 -- remove key
RANGE start_key end_key    -- retrieve all k,v pairs where start_key <= k <= end_key
SCAN                       -- retrieve all key-value pairs
```

**Patterns to follow:**
- Use pattern matching on the `Query` enum in the executor
- Keep parsing simple; no nested structures or complex grammar for POC

**Test scenarios:**
- Happy path: Parse and execute GET command
- Happy path: Parse and execute SET command
- Happy path: Parse and execute RANGE command
- Edge case: Parse GET with missing operand (should error)
- Edge case: Parse unknown operation
- Happy path: Execute SET followed by GET, verify data is retrievable
- Integration: Parse and execute multiple commands in sequence

**Verification:**
- Parser correctly parses all supported query formats
- Parser rejects malformed queries with clear error messages
- Executor correctly delegates to database and returns formatted results
- All query types produce expected output

---

### U6. CLI and Main Entry Point

**Goal:** Create a simple REPL so users can interact with the database, demonstrating all functionality.

**Requirements:** R5 (readable, demonstrable)

**Dependencies:** U5

**Files:**
- Modify: `src/main.rs` — implement REPL
- Modify: `Cargo.toml` — add any CLI dependencies if desired (optional, can use `std::io` only)

**Approach:**
- Implement a simple REPL that:
  - Prompts for user input (`> `)
  - Reads a query string
  - Parses and executes the query via the query layer
  - Prints results
  - Repeats until user enters `quit` or EOF
- Initialize a `Database` at startup, pointing to a `test.db` file
- Provide helpful startup message and error messages

**Patterns to follow:**
- Use `std::io` for reading from stdin
- Keep error messages user-friendly

**Test scenarios:**
- Test expectation: none -- REPL is UI, not unit-testable; integration tests cover the functionality

**Verification:**
- REPL starts and prompts for input
- User can execute SET, GET, RANGE, DELETE commands
- Results display correctly
- Program exits cleanly on `quit`

---

### U7. Integration Tests and Demo

**Goal:** Write comprehensive tests demonstrating all features end-to-end and create a demo scenario.

**Requirements:** R6 (test coverage), R5 (demonstrates concepts)

**Dependencies:** All units (U1–U6)

**Files:**
- Modify: `tests/integration_tests.rs` — comprehensive integration tests
- Create: `examples/demo.rs` (optional) — a demo scenario

**Approach:**
- Write integration tests that:
  - Test full workflows (e.g., create database → set 10 records → query range → delete some → verify results)
  - Test persistence (create database, add data, drop database, reopen, verify data intact)
  - Test edge cases (empty database, single record, large number of records)
- Optionally create an example that demonstrates the database with a sample dataset (e.g., key-value pairs representing user data)

**Patterns to follow:**
- Use `#[test]` attribute for integration tests
- Use setup/teardown (create temp directory for test databases)

**Test scenarios:**
- Full workflow: Set 5 records, query range that includes 3, verify results
- Persistence: Set records, close database, reopen, query range, verify all records intact
- Edge case: Empty range query (range with no matches)
- Edge case: Single record database, query range including that record
- Edge case: Large dataset (100+ records), random operations, verify consistency

**Verification:**
- All integration tests pass
- Tests demonstrate all major use cases
- Code is well-commented for educational clarity

---

## System-Wide Impact

**Interaction graph:** The layers interact linearly: Query → Executor → Database Facade → Index + Storage. No callbacks or middleware.

**Error propagation:** Errors bubble up from storage/index layers through the database facade to the query executor, which formats them for display.

**State lifecycle:** Database state persists across restarts via the storage file. Index is rebuilt from storage on startup.

**Unchanged invariants:** This POC does not introduce any breaking changes (it's greenfield). Future work (e.g., transactions) should preserve the simple get/set/range API.

---

## Risks & Dependencies

| Risk | Mitigation |
|------|------------|
| B-tree implementation complexity | Start with a simple implementation (small MAX_KEYS, no complex rebalancing); prioritize correctness over efficiency |
| File format not flexible for future changes | Document the record format clearly; note that future work may switch to page-oriented storage |
| Index rebuild on startup could be slow | For POC, this is acceptable; caching or persistent index is a follow-up optimization |
| Single-threaded limitation | Document this explicitly; multi-threading can be added later if needed |

---

## Sources & References

- **Origin document:** [docs/brainstorms/db-storage-engine-requirements.md](../brainstorms/db-storage-engine-requirements.md)
