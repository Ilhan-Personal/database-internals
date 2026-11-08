//! Storage abstraction layer.
//!
//! The [`StorageBackend`] trait defines a minimal interface for key-value storage.
//! Different implementations (in-memory, file-backed, etc.) can be plugged in.

pub mod memory;

pub use memory::InMemoryStorage;

use crate::Result;

/// Trait for pluggable storage backends.
///
/// This abstraction allows swapping storage implementations without changing
/// the B-tree or WAL logic.
pub trait StorageBackend: Send + Sync {
    /// Write a key-value pair. Overwrites existing key.
    fn write(&self, key: &[u8], value: &[u8]) -> Result<()>;

    /// Read a value by key. Returns None if key doesn't exist.
    fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    /// Delete a key. Idempotent (no error if key doesn't exist).
    fn delete(&self, key: &[u8]) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_storage_write_read() {
        let storage = InMemoryStorage::new();
        storage.write(b"key1", b"value1").unwrap();
        let result = storage.read(b"key1").unwrap();
        assert_eq!(result, Some(b"value1".to_vec()));
    }

    #[test]
    fn test_memory_storage_key_not_found() {
        let storage = InMemoryStorage::new();
        let result = storage.read(b"nonexistent").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_memory_storage_overwrite() {
        let storage = InMemoryStorage::new();
        storage.write(b"key1", b"value1").unwrap();
        storage.write(b"key1", b"value2").unwrap();
        let result = storage.read(b"key1").unwrap();
        assert_eq!(result, Some(b"value2".to_vec()));
    }

    #[test]
    fn test_memory_storage_delete() {
        let storage = InMemoryStorage::new();
        storage.write(b"key1", b"value1").unwrap();
        storage.delete(b"key1").unwrap();
        let result = storage.read(b"key1").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_memory_storage_delete_nonexistent() {
        let storage = InMemoryStorage::new();
        // Should not error
        storage.delete(b"nonexistent").unwrap();
    }
}
