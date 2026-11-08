//! An LSM-style store: writes land in a sorted in-memory [`memtable::Memtable`],
//! which is flushed to an immutable on-disk [`sstable`] file in level 0 once a
//! time interval or size threshold is hit.
//!
//! ```text
//!   put ──▶ Memtable (sorted, in memory)
//!              │  interval elapsed OR size >= threshold
//!              ▼
//!           L0/000001.sst, L0/000002.sst, ...  (sorted, immutable)
//!
//!   get ──▶ Memtable ──▶ L0 newest ──▶ ... ──▶ L0 oldest
//! ```

pub mod lsm;
pub mod memtable;
pub mod sstable;

pub use lsm::{Config, Lsm};
