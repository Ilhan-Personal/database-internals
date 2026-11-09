use std::sync::{Arc, RwLock};
use std::collections::BTreeMap;
use crate::Result;
use super::StorageBackend;

#[derive(Clone)]
pub struct InMemoryStorage {
    data: Arc<RwLock<BTreeMap<Vec<u8>, Vec<u8>>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        InMemoryStorage {
            data: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for InMemoryStorage {
    fn write(&self, key: &[u8], value: &[u8]) -> Result<()> {
        let mut data = self.data.write().unwrap();
        data.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let data = self.data.read().unwrap();
        Ok(data.get(key).cloned())
    }

    fn delete(&self, key: &[u8]) -> Result<()> {
        let mut data = self.data.write().unwrap();
        data.remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_writes_and_reads() {
        let storage = InMemoryStorage::new();
        for i in 0..100 {
            let key = format!("key{}", i).into_bytes();
            let value = format!("value{}", i).into_bytes();
            storage.write(&key, &value).unwrap();
        }

        for i in 0..100 {
            let key = format!("key{}", i).into_bytes();
            let value = format!("value{}", i).into_bytes();
            let result = storage.read(&key).unwrap();
            assert_eq!(result, Some(value));
        }
    }

    #[test]
    fn test_concurrent_reads() {
        use std::thread;

        let storage = InMemoryStorage::new();
        storage.write(b"shared_key", b"shared_value").unwrap();

        let mut handles = vec![];
        for _ in 0..10 {
            let storage_clone = storage.clone();
            let handle = thread::spawn(move || {
                let result = storage_clone.read(b"shared_key").unwrap();
                assert_eq!(result, Some(b"shared_value".to_vec()));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
