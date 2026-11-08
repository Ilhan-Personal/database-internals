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

use serde::{Deserialize, Serialize};

pub mod log;
pub mod recovery;

pub use log::WriteAheadLog;
pub use recovery::recover_from_log;

/// A log entry represents a single operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Sequence number for ordering and idempotence
    pub sequence: u64,
    /// Operation type: "INSERT" or "DELETE"
    pub op_type: String,
    /// Key involved in the operation
    pub key: Vec<u8>,
    /// Value (for INSERT operations)
    pub value: Option<Vec<u8>>,
    /// Timestamp (Unix seconds)
    pub timestamp: u64,
}

impl LogEntry {
    /// Create a new INSERT log entry.
    pub fn insert(sequence: u64, key: Vec<u8>, value: Vec<u8>) -> Self {
        LogEntry {
            sequence,
            op_type: "INSERT".to_string(),
            key,
            value: Some(value),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Create a new DELETE log entry.
    pub fn delete(sequence: u64, key: Vec<u8>) -> Self {
        LogEntry {
            sequence,
            op_type: "DELETE".to_string(),
            key,
            value: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_insert() {
        let entry = LogEntry::insert(1, b"key1".to_vec(), b"value1".to_vec());
        assert_eq!(entry.sequence, 1);
        assert_eq!(entry.op_type, "INSERT");
        assert_eq!(entry.key, b"key1");
        assert_eq!(entry.value, Some(b"value1".to_vec()));
    }

    #[test]
    fn test_log_entry_delete() {
        let entry = LogEntry::delete(2, b"key1".to_vec());
        assert_eq!(entry.sequence, 2);
        assert_eq!(entry.op_type, "DELETE");
        assert_eq!(entry.key, b"key1");
        assert_eq!(entry.value, None);
    }
}
