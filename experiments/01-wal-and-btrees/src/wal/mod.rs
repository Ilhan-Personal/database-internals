use serde::{Deserialize, Serialize};

pub mod log;
pub mod recovery;

pub use log::WriteAheadLog;
pub use recovery::recover_from_log;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub sequence: u64,
    pub op_type: String,
    pub key: Vec<u8>,
    pub value: Option<Vec<u8>>,
    pub timestamp: u64,
}

impl LogEntry {
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
