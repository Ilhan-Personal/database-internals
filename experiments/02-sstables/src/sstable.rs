//! Sorted String Table: an immutable on-disk file of key/value pairs in key order.
//!
//! File format, one record after another:
//!
//! ```text
//! [key_len: u32 LE][key bytes][value_len: u32 LE][value bytes] ...
//! ```

use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

const TEMP_EXTENSION: &str = "tmp";

/// Write `entries` (already in ascending key order) to a new file at `path`.
///
/// Write to a temp file, `sync_all`, then rename into place, so a crash
/// mid-write never leaves a half-written `.sst` behind.
pub fn write<'a>(
    path: &Path,
    entries: impl Iterator<Item = (&'a [u8], &'a [u8])>,
) -> io::Result<()> {
    let temp_path = path.with_extension(TEMP_EXTENSION);
    let mut writer = BufWriter::new(File::create(&temp_path)?);
    for (key, value) in entries {
        write_bytes(&mut writer, key)?;
        write_bytes(&mut writer, value)?;
    }
    let file = writer.into_inner().map_err(|e| e.into_error())?;
    file.sync_all()?;
    fs::rename(&temp_path, path)
}

fn write_bytes(writer: &mut impl Write, bytes: &[u8]) -> io::Result<()> {
    let len = u32::try_from(bytes.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "key or value over 4 GiB"))?;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(bytes)
}

/// Read one length-prefixed byte string. Returns `None` at a clean end of file.
fn read_bytes(reader: &mut impl Read) -> io::Result<Option<Vec<u8>>> {
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let mut bytes = vec![0u8; u32::from_le_bytes(len_buf) as usize];
    reader.read_exact(&mut bytes)?;
    Ok(Some(bytes))
}

/// A read handle on one SSTable file.
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

    /// Look up `key`. Because keys are sorted you can stop scanning as soon as
    /// you pass where `key` would be.
    pub fn get(&self, key: &[u8]) -> io::Result<Option<Vec<u8>>> {
        let mut reader = BufReader::new(File::open(&self.path)?);
        while let Some(record_key) = read_bytes(&mut reader)? {
            let value = read_bytes(&mut reader)?.ok_or_else(|| {
                io::Error::new(io::ErrorKind::UnexpectedEof, "record has a key but no value")
            })?;
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
