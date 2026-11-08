//! WAL recovery logic.

use crate::Result;
use crate::storage::StorageBackend;
use super::LogEntry;

/// Recover database state from WAL log entries.
///
/// Replays all log entries in order to reconstruct the last consistent state.
pub fn recover_from_log(storage: &dyn StorageBackend, entries: &[LogEntry]) -> Result<()> {
    for entry in entries {
        match entry.op_type.as_str() {
            "INSERT" => {
                if let Some(value) = &entry.value {
                    storage.write(&entry.key, value)?;
                }
            }
            "DELETE" => {
                storage.delete(&entry.key)?;
            }
            _ => {
                // Unknown operation type, skip
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::InMemoryStorage;

    #[test]
    fn test_recover_from_empty_log() {
        let storage = InMemoryStorage::new();
        let entries = vec![];
        recover_from_log(&storage, &entries).unwrap();
        // Should succeed without errors
    }

    #[test]
    fn test_recover_inserts() {
        let storage = InMemoryStorage::new();
        let entries = vec![
            LogEntry::insert(1, b"key1".to_vec(), b"value1".to_vec()),
            LogEntry::insert(2, b"key2".to_vec(), b"value2".to_vec()),
        ];

        recover_from_log(&storage, &entries).unwrap();

        assert_eq!(storage.read(b"key1").unwrap(), Some(b"value1".to_vec()));
        assert_eq!(storage.read(b"key2").unwrap(), Some(b"value2".to_vec()));
    }

    #[test]
    fn test_recover_deletes() {
        let storage = InMemoryStorage::new();
        storage.write(b"key1", b"value1").unwrap();

        let entries = vec![LogEntry::delete(1, b"key1".to_vec())];

        recover_from_log(&storage, &entries).unwrap();

        assert_eq!(storage.read(b"key1").unwrap(), None);
    }

    #[test]
    fn test_recover_mixed_operations() {
        let storage = InMemoryStorage::new();
        let entries = vec![
            LogEntry::insert(1, b"key1".to_vec(), b"value1".to_vec()),
            LogEntry::insert(2, b"key2".to_vec(), b"value2".to_vec()),
            LogEntry::delete(3, b"key1".to_vec()),
            LogEntry::insert(4, b"key3".to_vec(), b"value3".to_vec()),
        ];

        recover_from_log(&storage, &entries).unwrap();

        assert_eq!(storage.read(b"key1").unwrap(), None);
        assert_eq!(storage.read(b"key2").unwrap(), Some(b"value2".to_vec()));
        assert_eq!(storage.read(b"key3").unwrap(), Some(b"value3".to_vec()));
    }
}
