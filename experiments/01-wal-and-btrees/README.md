# 01 — WAL and B-trees

**Status:** In progress
**Started:** 2026-05-10

## Goal

Implement three ideas from *Database Internals*, Chapter 6:

- **Write-ahead log (WAL):** log each operation before applying it, so state can be recovered after a crash.
- **Copy-on-write B-tree:** a B-tree with multiple versions, so readers and writers don't block each other.
- **Lazy B-tree:** buffer writes and apply them in batches to get more throughput.

## What I built

- `src/storage/`: storage trait and an in-memory backend
- `src/wal/`: log entries, appending to the log, recovery
- `src/btree/`, `src/buffer/`: stubs, not started yet

```bash
cargo test -p database_internals
cargo run -p database_internals --example wal_basic
```

Plans and brainstorms are in [`docs/`](docs/).

## Results

_TBD_

## Learnings

_TBD_

## References

- Alex Petrov, *Database Internals*, Chapter 6
