pub mod memory;

pub use memory::InMemoryStorage;

use crate::Result;

pub trait StorageBackend: Send + Sync {
    fn write(&self, key: &[u8], value: &[u8]) -> Result<()>;

    fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

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
        storage.delete(b"nonexistent").unwrap();
    }
}
