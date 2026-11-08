//! Ties the memtable and level-0 SSTables together.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::memtable::Memtable;
use crate::sstable::{self, SsTable};

pub const LEVEL0_DIR: &str = "L0";
const SSTABLE_EXTENSION: &str = "sst";

/// When the memtable gets flushed to level 0 — whichever is hit first.
pub struct Config {
    pub flush_interval: Duration,
    pub flush_size_bytes: usize,
}

pub struct Lsm {
    dir: PathBuf,
    config: Config,
    memtable: Memtable,
    last_flush: Instant,
    /// Level-0 tables, oldest first. L0 tables can overlap in key range,
    /// so reads must check newest to oldest.
    level0: Vec<SsTable>,
    next_table_id: u64,
}

impl Lsm {
    /// Open (or create) a store in `dir`. Load any existing `L0/*.sst`
    /// files so data survives a restart.
    pub fn open(dir: impl Into<PathBuf>, config: Config) -> io::Result<Self> {
        let dir = dir.into();
        let level0_dir = dir.join(LEVEL0_DIR);
        fs::create_dir_all(&level0_dir)?;

        let mut table_ids = Vec::new();
        for entry in fs::read_dir(&level0_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some(SSTABLE_EXTENSION) {
                continue;
            }
            if let Some(id) = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|stem| stem.parse::<u64>().ok())
            {
                table_ids.push(id);
            }
        }
        table_ids.sort_unstable();

        let mut level0 = Vec::with_capacity(table_ids.len());
        for &id in &table_ids {
            level0.push(SsTable::open(&table_path(&dir, id))?);
        }

        Ok(Lsm {
            dir,
            config,
            memtable: Memtable::new(),
            last_flush: Instant::now(),
            level0,
            next_table_id: table_ids.last().map_or(1, |id| id + 1),
        })
    }

    /// Write to the memtable, then flush if `should_flush`.
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> io::Result<()> {
        self.memtable.put(key, value);
        if self.should_flush() {
            self.flush()?;
        }
        Ok(())
    }

    /// Memtable first, then level 0 from newest to oldest.
    pub fn get(&self, key: &[u8]) -> io::Result<Option<Vec<u8>>> {
        if let Some(value) = self.memtable.get(key) {
            return Ok(Some(value.to_vec()));
        }
        for table in self.level0.iter().rev() {
            if let Some(value) = table.get(key)? {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }

    /// True when the interval has elapsed or the size threshold is reached.
    fn should_flush(&self) -> bool {
        self.last_flush.elapsed() >= self.config.flush_interval
            || self.memtable.size_bytes() >= self.config.flush_size_bytes
    }

    /// Write the memtable to `L0/<next_table_id>.sst`, add it to `level0`,
    /// swap in an empty memtable, and reset `last_flush`.
    fn flush(&mut self) -> io::Result<()> {
        let path = table_path(&self.dir, self.next_table_id);
        // Write before swapping: if the write fails, the data is still in the memtable.
        sstable::write(&path, self.memtable.iter())?;
        self.level0.push(SsTable::open(&path)?);
        self.memtable = Memtable::new();
        self.next_table_id += 1;
        self.last_flush = Instant::now();
        Ok(())
    }
}

fn table_path(dir: &Path, id: u64) -> PathBuf {
    dir.join(LEVEL0_DIR)
        .join(format!("{id:06}.{SSTABLE_EXTENSION}"))
}
