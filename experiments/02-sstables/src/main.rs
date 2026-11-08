use std::fs;
use std::io;
use std::path::Path;
use std::thread;
use std::time::Duration;

use sstables::lsm::LEVEL0_DIR;
use sstables::memtable::Memtable;
use sstables::{Config, Lsm};

fn main() -> io::Result<()> {
    memtable_demo();
    lsm_demo()
}

fn memtable_demo() {
    println!("== memtable");
    let mut mem = Memtable::new();
    println!("empty at start: {}", mem.is_empty());

    mem.put(b"cherry".to_vec(), b"red".to_vec());
    mem.put(b"apple".to_vec(), b"green".to_vec());
    mem.put(b"banana".to_vec(), b"yellow".to_vec());
    println!("size after 3 puts: {} bytes", mem.size_bytes());

    mem.put(b"apple".to_vec(), b"red".to_vec());
    println!("size after overwriting apple (expect 2 bytes less): {} bytes", mem.size_bytes());

    println!("get apple: {:?}", mem.get(b"apple").map(String::from_utf8_lossy));
    println!("get durian: {:?}", mem.get(b"durian").map(String::from_utf8_lossy));

    println!("in key order (expect apple, banana, cherry):");
    for (key, value) in mem.iter() {
        println!("  {} = {}", String::from_utf8_lossy(key), String::from_utf8_lossy(value));
    }
}

fn lsm_demo() -> io::Result<()> {
    println!("\n== lsm");
    let dir = std::env::temp_dir().join("sstables-demo");
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    let config = || Config {
        flush_interval: Duration::from_millis(500),
        flush_size_bytes: 32,
    };

    let mut db = Lsm::open(&dir, config())?;
    db.put(b"apple".to_vec(), b"green".to_vec())?;
    db.put(b"banana".to_vec(), b"yellow".to_vec())?;
    println!("after 2 small puts (expect no files): {:?}", level0_files(&dir)?);

    db.put(b"cherry".to_vec(), b"red-and-shiny".to_vec())?;
    println!("after crossing 32 bytes (expect 1 file): {:?}", level0_files(&dir)?);

    db.put(b"apple".to_vec(), b"red".to_vec())?;
    thread::sleep(Duration::from_millis(600));
    db.put(b"date".to_vec(), b"brown".to_vec())?;
    println!("after waiting past 500ms (expect 2 files): {:?}", level0_files(&dir)?);

    println!("get apple (expect newer \"red\"): {:?}", as_text(db.get(b"apple")?));
    println!("get cherry (from first file): {:?}", as_text(db.get(b"cherry")?));
    println!("get durian (expect None): {:?}", as_text(db.get(b"durian")?));

    drop(db);
    let db = Lsm::open(&dir, config())?;
    println!("after reopening, get banana: {:?}", as_text(db.get(b"banana")?));
    Ok(())
}

fn level0_files(dir: &Path) -> io::Result<Vec<String>> {
    let mut names: Vec<String> = fs::read_dir(dir.join(LEVEL0_DIR))?
        .map(|entry| entry.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect::<io::Result<_>>()?;
    names.sort();
    Ok(names)
}

fn as_text(value: Option<Vec<u8>>) -> Option<String> {
    value.map(|v| String::from_utf8_lossy(&v).into_owned())
}
