//! # Database Internals: Learning Implementation
//!
//! A educational Rust implementation of three key database optimization concepts from
//! *Database Internals* Chapter 6:
//!
//! - **Write Ahead Logging (WAL)**: Durability mechanism that logs operations before
//!   executing them, enabling crash recovery.
//! - **Copy-on-Write B-Trees**: Multi-version data structure allowing concurrent readers
//!   and writers without blocking.
//! - **Lazy Trees**: Buffer-based write optimization that batches operations for improved
//!   throughput.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │  Examples & Integration Tests               │
//! ├─────────────────────────────────────────────┤
//! │  Lazy Tree | CoW B-Tree | Standard B-Tree   │
//! ├─────────────────────────────────────────────┤
//! │  WAL System                                 │
//! ├─────────────────────────────────────────────┤
//! │  Storage Trait + In-Memory Backend          │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! ## Getting Started
//!
//! Start with the [`storage`] module to understand the storage abstraction,
//! then explore [`wal`] for durability, and finally [`btree`] for the three
//! data structure variants.
//!
//! Run examples:
//!
//! ```bash
//! cargo run --example wal_basic
//! cargo run --example btree_operations
//! cargo run --example cow_concurrent
//! cargo run --example lazy_tree_bulk
//! cargo run --example full_scenario
//! ```

pub mod error;
pub mod storage;
pub mod wal;
pub mod btree;
pub mod buffer;

pub use error::{DbError, Result};
pub use storage::StorageBackend;
