//! In-memory write buffer that keeps keys sorted.

use std::collections::BTreeMap;

/// Sorted in-memory key/value buffer.
///
/// A `BTreeMap` keeps keys ordered, so flushing is just iterating in order.
pub struct Memtable {
    entries: BTreeMap<Vec<u8>, Vec<u8>>,
    /// Approximate bytes held (keys + values), used for the size threshold.
    size_bytes: usize,
}

impl Memtable {
    pub fn new() -> Self {
        Memtable {
            entries: BTreeMap::new(),
            size_bytes: 0,
        }
    }

    /// Insert or overwrite `key`. Keep `size_bytes` in sync — remember an
    /// overwrite replaces the old value's bytes rather than adding to them.
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        let key_len = key.len();
        let value_len = value.len();
        match self.entries.insert(key, value) {
            Some(old_value) => self.size_bytes = self.size_bytes + value_len - old_value.len(),
            None => self.size_bytes += key_len + value_len,
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.entries.get(key).map(|value| value.as_slice())
    }

    pub fn size_bytes(&self) -> usize {
        self.size_bytes
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Entries in ascending key order — what gets written to an SSTable.
    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &[u8])> {
        self.entries
            .iter()
            .map(|(key, value)| (key.as_slice(), value.as_slice()))
    }
}
