use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, Seek, Write};
use std::path::Path;

use crate::record::{self, Record};

pub struct CommitLog {
    file: File,
}

impl CommitLog {
    pub fn open(path: &Path) -> io::Result<(Self, Vec<Record>)> {
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path)?;

        let mut records = Vec::new();
        let mut reader = BufReader::new(&file);
        let mut complete_len = 0;
        loop {
            match record::read(&mut reader) {
                Ok(Some(record)) => {
                    records.push(record);
                    complete_len = reader.stream_position()?;
                }
                Ok(None) => break,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
        }
        drop(reader);

        file.set_len(complete_len)?;
        file.sync_data()?;
        if let Some(dir) = path.parent() {
            File::open(dir)?.sync_all()?;
        }
        Ok((CommitLog { file }, records))
    }

    pub fn append(&mut self, key: &[u8], value: Option<&[u8]>) -> io::Result<()> {
        let mut buf = Vec::new();
        record::write(&mut buf, key, value)?;
        self.file.write_all(&buf)?;
        self.file.sync_data()
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.file.set_len(0)?;
        self.file.sync_data()
    }
}
