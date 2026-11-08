# Database Storage Engine in Rust — Requirements

**Status:** Ready for planning  
**Scope:** Proof-of-concept  
**Timeline:** Few sessions  

---

## Problem & Learning Goals

Build a **database storage engine from scratch in Rust** to understand how databases work internally — specifically how data is organized, persisted, indexed, and queried. This is a learning project anchored to fundamental concepts: data structures (B-trees), disk I/O, and query execution.

---

## What We're Building

A **key-value storage engine** with three core capabilities:

### 1. Basic Key-Value Storage + Disk Persistence
- Store key-value pairs that survive process restarts
- Simple operations: `get(key)`, `set(key, value)`, `delete(key)`
- Data persisted to disk in a format that's efficient to read back
- In-memory working set for active operations

### 2. B-Tree Indexing
- B-tree structure for ordered key retrieval and range queries
- Support range queries: `get_range(start_key, end_key)` 
- Efficient key lookups without scanning the entire dataset
- Ordered iteration over keys

### 3. Query Execution
- Parse and execute simple queries (e.g., SQL-like or custom format)
- Execute basic operations: SELECT, WHERE on keys, simple filtering
- Returns results as collections (Vec or iterator)
- Focus on correctness, not optimization or query planning

---

## What's Out of Scope (For Now)

- **Transactions & ACID** — Single-statement operations only; no multi-statement atomicity or rollback
- **Concurrent access** — Single-threaded for now; locking/synchronization deferred
- **Query planning/optimization** — Execute queries naively; no cost-based optimizer
- **Advanced indexing** — Only B-tree; skip hash tables, full-text search, secondary indexes
- **Network/RPC** — Local in-process only; no server mode or remote protocol

---

## Success Criteria

- [ ] Key-value operations (get/set/delete) work correctly
- [ ] Data persists to disk and is correctly read back after restart
- [ ] B-tree supports ordered retrieval and range queries
- [ ] Simple queries can be parsed and executed
- [ ] Code is readable and demonstrates the concepts clearly
- [ ] Basic tests verify core functionality

---

## Architecture Outline

**Components:**
- **Storage Layer** — Disk I/O, file format, serialization
- **Index Layer** — B-tree data structure and operations
- **Query Layer** — Parser and executor for simple queries
- **In-Memory Buffer** — Active working set in RAM

**Data Flow:**
1. Query arrives → parsed into operations
2. Operations consult B-tree index
3. Index points to disk locations
4. Storage layer fetches data from disk or buffer
5. Results returned to caller

---

## Assumptions & Dependencies

- **Language:** Rust (stability, learning value, performance)
- **No external database libraries** — Build core functionality; OK to use serialization libs (serde) or parsing helpers
- **Single process, single thread** — Simplifies I/O and state management
- **File-based storage** — On-disk format of choice (binary, simple record format)

---

## Key Questions / Open Decisions

1. **File format** — Binary records, page-oriented, or something else?
2. **B-tree page size** — In-memory nodes vs disk-backed pages?
3. **Query language** — Simple SQL-like syntax or custom DSL?
4. **Error handling** — Strict validation or permissive defaults?

These will be resolved in planning once we outline the implementation strategy.

---

## Next Steps

→ **Plan the implementation** — Define module structure, data layouts, and build phases  
→ **Start with storage layer** — Get disk I/O working first, then build indexing on top
