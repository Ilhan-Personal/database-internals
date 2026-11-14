use std::io::{self, Read, Write};

const KIND_PUT: u8 = 0;
const KIND_DELETE: u8 = 1;

const RECORD_LEN_BYTES: usize = 4;
pub type Record = (Vec<u8>, Option<Vec<u8>>);

pub fn write(writer: &mut impl Write, key: &[u8], value: Option<&[u8]>) -> io::Result<()> {
    match value {
        Some(value) => {
            writer.write_all(&[KIND_PUT])?;
            write_bytes(writer, key)?;
            write_bytes(writer, value)
        }
        None => {
            writer.write_all(&[KIND_DELETE])?;
            write_bytes(writer, key)
        }
    }
}

pub fn read(reader: &mut impl Read) -> io::Result<Option<Record>> {
    let mut kind = [0u8; 1];
    match reader.read_exact(&mut kind) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let key = read_bytes(reader)?;
    let value = match kind[0] {
        KIND_PUT => Some(read_bytes(reader)?),
        KIND_DELETE => None,
        other => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown record kind {other}"),
            ))
        }
    };
    Ok(Some((key, value)))
}

fn write_bytes(writer: &mut impl Write, bytes: &[u8]) -> io::Result<()> {
    let len = u32::try_from(bytes.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "key or value over 4 GiB"))?;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(bytes)
}

fn read_bytes(reader: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut len_buf = [0u8; RECORD_LEN_BYTES];
    reader.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf);
    let mut bytes = Vec::new();
    reader.take(len as u64).read_to_end(&mut bytes)?;
    if bytes.len() != len as usize {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "record cut off"));
    }
    Ok(bytes)
}
