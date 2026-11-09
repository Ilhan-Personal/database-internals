use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::commit_log::CommitLog;
use crate::memtable::Memtable;
use crate::sstable::{self, SsTable};

pub const LEVEL0_DIR: &str = "L0";
pub const COMMIT_LOG_FILE: &str = "commit.log";
const SSTABLE_EXTENSION: &str = "sst";

pub struct Config {
    pub flush_interval: Duration,
    pub flush_size_bytes: usize,
}

pub struct Lsm {
    dir: PathBuf,
    config: Config,
    log: CommitLog,
    memtable: Memtable,
    last_flush: Instant,
    level0: Vec<SsTable>,
    next_table_id: u64,
}

impl Lsm {
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

        let (log, records) = CommitLog::open(&dir.join(COMMIT_LOG_FILE))?;
        let mut memtable = Memtable::new();
        for (key, value) in records {
            match value {
                Some(value) => memtable.put(key, value),
                None => memtable.delete(key),
            }
        }

        let mut lsm = Lsm {
            dir,
            config,
            log,
            memtable,
            last_flush: Instant::now(),
            level0,
            next_table_id: table_ids.last().map_or(1, |id| id + 1),
        };
        lsm.flush_if_needed()?;
        Ok(lsm)
    }

    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> io::Result<()> {
        self.log.append(&key, Some(&value))?;
        self.memtable.put(key, value);
        self.flush_if_needed()
    }

    pub fn delete(&mut self, key: Vec<u8>) -> io::Result<()> {
        self.log.append(&key, None)?;
        self.memtable.delete(key);
        self.flush_if_needed()
    }

    pub fn get(&self, key: &[u8]) -> io::Result<Option<Vec<u8>>> {
        if let Some(value) = self.memtable.get(key) {
            return Ok(value.map(<[u8]>::to_vec));
        }
        for table in self.level0.iter().rev() {
            if let Some(value) = table.get(key)? {
                return Ok(value);
            }
        }
        Ok(None)
    }

    fn flush_if_needed(&mut self) -> io::Result<()> {
        if self.should_flush() {
            self.flush()?;
        }
        Ok(())
    }

    fn should_flush(&self) -> bool {
        self.last_flush.elapsed() >= self.config.flush_interval
            || self.memtable.size_bytes() >= self.config.flush_size_bytes
    }

    fn flush(&mut self) -> io::Result<()> {
        let path = table_path(&self.dir, self.next_table_id);
        sstable::write(&path, self.memtable.iter())?;
        self.level0.push(SsTable::open(&path)?);
        self.log.clear()?;
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
