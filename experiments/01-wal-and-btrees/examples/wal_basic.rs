//! Minimal WAL demo: append operations and replay into storage.

use database_internals::storage::InMemoryStorage;
use database_internals::StorageBackend;
use database_internals::wal::{recover_from_log, WriteAheadLog};

fn main() {
    let mut wal = WriteAheadLog::new().expect("scratch WAL temp file");
    let storage = InMemoryStorage::new();

    wal.log_insert(b"hello".to_vec(), b"world".to_vec()).unwrap();
    recover_from_log(&storage, &wal.entries()).unwrap();

    let got = storage.read(b"hello").unwrap().expect("replay wrote key");
    println!("{}", String::from_utf8_lossy(&got));
}
