use std::collections::BTreeMap;

pub struct Memtable {
    entries: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    size_bytes: usize,
}

impl Memtable {
    pub fn new() -> Self {
        Memtable {
            entries: BTreeMap::new(),
            size_bytes: 0,
        }
    }

    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.insert(key, Some(value));
    }

    pub fn delete(&mut self, key: Vec<u8>) {
        self.insert(key, None);
    }

    fn insert(&mut self, key: Vec<u8>, value: Option<Vec<u8>>) {
        let key_len = key.len();
        let value_len = value.as_ref().map_or(0, Vec::len);
        match self.entries.insert(key, value) {
            Some(old_value) => {
                self.size_bytes = self.size_bytes + value_len - old_value.map_or(0, |v| v.len())
            }
            None => self.size_bytes += key_len + value_len,
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<Option<&[u8]>> {
        self.entries.get(key).map(|value| value.as_deref())
    }

    pub fn size_bytes(&self) -> usize {
        self.size_bytes
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&[u8], Option<&[u8]>)> {
        self.entries
            .iter()
            .map(|(key, value)| (key.as_slice(), value.as_deref()))
    }
}
