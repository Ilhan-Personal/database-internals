# Database Internals

My notes and experiments from learning how databases work by building the pieces myself, in Rust.

Each experiment is a small crate under [`experiments/`](experiments/) with its own README that records
what I set out to learn, what I built, and what I took away from it.

## Experiments

| #  | Experiment                                          | Topics                                  | Status         |
|----|-----------------------------------------------------|-----------------------------------------|----------------|
| 01 | [WAL and B-trees](experiments/01-wal-and-btrees/)   | Write-ahead log, CoW B-tree, lazy B-tree | In progress   |
| 02 | [SSTables](experiments/02-sstables/)                | Memtable, level-0 SSTables, LSM         | In progress    |

## Layout

```text
.
├── Cargo.toml              # Cargo workspace; every folder in experiments/ is a member
├── experiments/
│   └── NN-topic/           # One crate per experiment
│       ├── README.md       # Goal, what I built, learnings, references
│       ├── Cargo.toml
│       ├── src/
│       ├── examples/
│       └── docs/           # Plans and brainstorms for this experiment
├── docs/
│   └── research/           # Notes that apply across experiments
└── templates/experiment/   # README template for new experiments
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
