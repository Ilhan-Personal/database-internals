# Database Internals

My notes and experiments from learning how databases work by building the pieces myself, in Rust.

Each experiment is a small crate under [`experiments/`](experiments/) with its own README that records
what I set out to learn, what I built, and what I took away from it.

## Experiments

| #  | Experiment                                          | Topics                                  | Status         |
|----|-----------------------------------------------------|-----------------------------------------|----------------|
| 01 | [WAL and B-trees](experiments/01-wal-and-btrees/)   | Write-ahead log, CoW B-tree, lazy B-tree | done   |
| 02 | [SSTables](experiments/02-sstables/)                | Memtable, level-0 SSTables, LSM         | done    |

```

## Starting a new experiment

```bash
cargo new --lib experiments/02-sstables --name sstables
cp templates/experiment/README.md experiments/02-sstables/
# Fill in the README, then add a row to the table above.
```

## Running

```bash
cargo test                                   # every experiment
cargo test -p database_internals             # one experiment
cargo run -p database_internals --example wal_basic
```
