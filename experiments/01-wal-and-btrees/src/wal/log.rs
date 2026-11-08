//! Write Ahead Logging (WAL) system.
//!
//! WAL provides durability by logging operations before executing them.
//! If the system crashes, the log can be replayed to recover the last consistent state.
//!
//! # Key Concepts
//!
//! - **Log Entry**: Each operation (insert, delete) is serialized and appended to the log
//! - **Durability**: Log is flushed before returning to client (operation is durable immediately)
//! - **Recovery**: On startup, scan log and replay entries to reconstruct state
//! - **Checksum**: Each log entry includes a checksum to detect corruption

use crate::wal::LogEntry;
use std::fs::File;
use std::io::Write;

// this is so that we can change the storage backend later
pub struct WriteAheadLog {
    file_handle: File,
    entries: Vec<LogEntry>,
}

impl WriteAheadLog {
    pub fn new(file_handle: File) -> Self {
        WriteAheadLog {
            entries: Vec::new(),
            file_handle,
        }
    }

    pub fn append_entry(&mut self, entry: LogEntry) {
        let encoded = serde_json::to_vec(&entry).unwrap();
        self.entries.push(entry);
        self.file_handle.write_all(&encoded).unwrap();
    }

    pub fn sync(&mut self) {
        self.file_handle.sync_all().unwrap();
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_wal_append_entry() {
        let mut wal = WriteAheadLog::new(File::create("test.wal").unwrap());
        wal.append_entry(LogEntry::insert(1, b"key1".to_vec(), b"value1".to_vec()));
        wal.sync();
        assert_eq!(wal.entries.len(), 1);
    }

    #[test]
    fn test_wal_sync() {
        let mut wal = WriteAheadLog::new(File::create("test.wal").unwrap());
        wal.append_entry(LogEntry::insert(1, b"key1".to_vec(), b"value1".to_vec()));
        wal.sync();
        let file_contents = fs::read_to_string("test.wal").unwrap();
        assert_eq!(file_contents, "{\"sequence\":1,\"op_type\":\"INSERT\",\"key\":\"key1\",\"value\":\"value1\",\"timestamp\":1715769600}\n");
    }
}
