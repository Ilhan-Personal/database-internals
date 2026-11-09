use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::{Path, PathBuf};

use crate::record;

const TEMP_EXTENSION: &str = "tmp";

pub fn write<'a>(
    path: &Path,
    entries: impl Iterator<Item = (&'a [u8], Option<&'a [u8]>)>,
) -> io::Result<()> {
    let temp_path = path.with_extension(TEMP_EXTENSION);
    let mut writer = BufWriter::new(File::create(&temp_path)?);
    for (key, value) in entries {
        record::write(&mut writer, key, value)?;
    }
    let file = writer.into_inner().map_err(|e| e.into_error())?;
    file.sync_all()?;
    fs::rename(&temp_path, path)?;
    if let Some(dir) = path.parent() {
        File::open(dir)?.sync_all()?;
    }
    Ok(())
}

pub struct SsTable {
    path: PathBuf,
}

impl SsTable {
    pub fn open(path: &Path) -> io::Result<Self> {
        File::open(path)?;
        Ok(SsTable {
            path: path.to_path_buf(),
        })
    }

    pub fn get(&self, key: &[u8]) -> io::Result<Option<Option<Vec<u8>>>> {
        let mut reader = BufReader::new(File::open(&self.path)?);
        while let Some((record_key, value)) = record::read(&mut reader)? {
            if record_key.as_slice() == key {
                return Ok(Some(value));
            }
            if record_key.as_slice() > key {
                return Ok(None);
            }
        }
        Ok(None)
    }
}
