use database_internals::storage::InMemoryStorage;
use database_internals::StorageBackend;
use database_internals::wal::{recover_from_log, WriteAheadLog};

fn main() {
    let mut wal = WriteAheadLog::new().expect("scratch WAL temp file");
    let storage = InMemoryStorage::new();

    wal.log_insert(b"a".to_vec(), b"1".to_vec()).unwrap();
    wal.log_insert(b"b".to_vec(), b"2".to_vec()).unwrap();
    wal.log_delete(b"a".to_vec()).unwrap();

    recover_from_log(&storage, &wal.entries()).unwrap();

    assert!(storage.read(b"a").unwrap().is_none());
    assert_eq!(storage.read(b"b").unwrap().unwrap(), b"2");
    println!("full_scenario ok: final state consistent with log replay");
}
