# Building a Tiny LSM Storage Engine in Rust: Memtables, SSTables, Commit Logs and Tombstones

*Draft for Medium. Diagrams are in `diagrams/`: upload the `.png` files where marked. The `.excalidraw` sources open at excalidraw.com if you want to edit them.*

---

I've been learning how databases work by building the pieces myself. This post covers the storage engine design behind Bigtable, LevelDB, RocksDB and Cassandra: the **log-structured merge tree**, or LSM tree.

By the end we'll have a small key-value store, written in Rust with only the standard library, that:

- keeps recent writes sorted in memory (the **memtable**)
- flushes them to immutable sorted files on disk (**SSTables**)
- survives crashes, because every write goes to a **commit log** first
- supports deletes, using **tombstones**

The code is on GitHub: [github.com/Ilhan-Personal/database-internals](https://github.com/Ilhan-Personal/database-internals), in `experiments/02-sstables`.

## The big idea

Most of the cost of writing to disk comes from jumping around: updating a record in place means finding where it lives and rewriting it there. An LSM tree avoids that completely. It never updates anything in place:

1. Writes go into a sorted structure **in memory**.
2. When that fills up, it's written to disk in one go, as a new sorted file that is never modified again.
3. Reads check memory first, then the files from newest to oldest.

Every disk write is a sequential append. The price is paid on reads, which may have to look in several places, and that's the trade-off LSM trees are built around.

![The write path](diagrams/01-write-path.png)

## The memtable: a sorted map with a size counter

The memtable is where writes land. It needs to keep keys sorted, because when it's flushed the file must be in key order. Rust's `BTreeMap` does exactly that: it stores keys in a balanced tree, so iterating over it always visits keys from smallest to largest. With a `HashMap` you'd have to collect and sort before every flush.

```rust
pub struct Memtable {
    /// A `None` value marks a deleted key (a tombstone).
    entries: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    /// Approximate bytes held (keys + values), used for the size threshold.
    size_bytes: usize,
}
```

Keys and values are raw bytes (`Vec<u8>`), compared byte by byte. That gives some surprising orders: `"Zebra"` sorts before `"apple"` because uppercase letters have smaller byte values, and `"10"` sorts before `"9"`. If you store numbers as keys, encode them **big-endian** so byte order matches numeric order.

The memtable also tracks its approximate size, so it knows when to flush. There's a trap here: overwriting a key must *replace* the old value's bytes, not add to them, or writing the same key a thousand times would count it a thousand times:

```rust
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
```

`BTreeMap::insert` hands back the old value if there was one, which is exactly what we need. The lengths are read *before* the insert because `insert` takes ownership of `key` and `value`, and Rust won't let us use them afterwards.

### When to flush

The memtable is flushed when **either** of two limits is hit:

- **size**: it holds more than `flush_size_bytes`
- **time**: `flush_interval` has passed since the last flush

The time limit stops a quiet store from keeping data in memory indefinitely. Bigtable flushes on size only; I added the interval while experimenting.

## SSTables: sorted, immutable files

An SSTable (Sorted String Table) is just the memtable's entries written to a file in key order. I used a simple length-prefixed format, and the same format is reused for the commit log later:

![Record format](diagrams/02-record-format.png)

Every key and value is preceded by its length as a 4-byte little-endian `u32`. You can't use a separator like a comma or newline, because keys and values are arbitrary bytes and could contain it. A length works for any content. It also sets the limits: 4 bytes can hold numbers up to 2³² − 1, so a key or value can be at most about 4 GiB. That's far more than anyone needs; Bigtable caps row keys at 64 KB, and says 10–100 bytes is typical.

The write code uses `u32::try_from` rather than `as u32`. A plain cast would silently wrap a 5 GiB length to a smaller number and produce a corrupt file; `try_from` returns an error instead.

### Writing a file without ever leaving half of it behind

A crash in the middle of writing an SSTable must not leave a half-written file that looks real. So the file is written in four steps:

1. Write everything to `000003.tmp`.
2. `fsync` the file, so its bytes are actually on the disk and not just in the OS cache.
3. `rename` it to `000003.sst`. A rename is atomic: other code sees either the old name or the new one, never something in between.
4. `fsync` the **directory**, because the rename itself is a change to the directory, and it isn't durable until that's synced too.

```rust
let file = writer.into_inner().map_err(|e| e.into_error())?;
file.sync_all()?;
fs::rename(&temp_path, path)?;
if let Some(dir) = path.parent() {
    File::open(dir)?.sync_all()?;
}
```

Step 4 is easy to miss: without it, the file's contents can be safely on disk while its new name isn't.

### Reading: newest first, first hit wins

Each flush creates a new file, and files can overlap: `apple` might be in `000001.sst` and again, with a newer value, in `000002.sst`. So reads check the memtable, then the SSTables from **newest to oldest**, and stop at the first one that knows the key.

Within a file, keys are sorted, so the scan can stop early: if we're looking for `banana` and reach `cherry`, `banana` isn't in this file.

## Problem 1: a crash loses everything in the memtable

With just a memtable and SSTables, any write that hasn't been flushed yet lives only in memory. Kill the process and it's gone, even though `put` already returned `Ok`.

The fix is the oldest trick in databases: a **commit log**, also called a write-ahead log. Before a write touches the memtable, it's appended to a log file and synced to disk:

```rust
pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> io::Result<()> {
    self.log.append(&key, Some(&value))?;   // 1. on disk first
    self.memtable.put(key, value);          // 2. then in memory
    self.flush_if_needed()
}
```

On startup, the store reads the log from start to finish and replays every record into a fresh memtable, rebuilding exactly what was in memory before the crash.

![Crash recovery](diagrams/04-crash-recovery.png)

Three details took some thought.

**A partial record at the end of the log.** If the process crashes in the middle of an append, the log ends with a fragment of a record. That write was never acknowledged, because `put` hadn't returned yet, so it's safe to drop. Recovery reads records until the file ends part-way through one, then truncates the file back to the last complete record.

This only handles a record that's *cut short*. If a crash left garbage bytes instead (say, a length field that reads as 3 billion), the format has no way to tell garbage from real data. Two defences help: the reader never allocates more memory than there are bytes left in the file, so a bogus length can't trigger a 4 GiB allocation; and the real fix, a checksum per record, is on the to-do list below.

**When to empty the log.** Once the memtable has been flushed, its records are safely in an SSTable and the log can be emptied. The order matters: the SSTable must be fully durable (file synced, renamed, directory synced) *before* the log is cleared. If it were the other way around, a crash between the two steps would lose data.

**What if we crash between the two steps anyway?** Then the log still holds records that are already in the newest SSTable, and they get replayed into the memtable on restart. That's harmless: replaying them rebuilds the same entries, and the next flush writes them again. Making recovery safe to repeat is much easier than making every step perfectly ordered.

The same directory rule from SSTables applies here: when `commit.log` is first created, the directory is synced too, or a crash could lose the file's name along with the writes inside it.

The trade-off is speed. Syncing to disk on every write is slow, since each `put` waits for the disk. Real systems use **group commit**: they batch many writes into one sync. Bigtable does this too.

## Problem 2: how do you delete from a file you can't change?

SSTables are immutable. If `cherry` is in `000001.sst`, you can't remove it from that file. And just removing it from the memtable isn't enough: the next read would miss the memtable, fall through to `000001.sst`, and find the old value again.

The answer is to write a **tombstone**, a record that says "this key is deleted." In the memtable that's a `None` value, and in a file it's a record with the delete flag and no value (the second row in the record-format diagram above).

The read rule doesn't change: newest to oldest, first hit wins. Now a hit can be a tombstone, and a tombstone means "stop, it's deleted":

![Read path with a tombstone](diagrams/03-read-path.png)

In Rust this fits neatly into nested `Option`s. A lookup in one table returns:

- `None`: this table has never heard of the key, so keep looking
- `Some(None)`: this table has a tombstone, so the key is deleted and we stop
- `Some(Some(value))`: found it

```rust
pub fn get(&self, key: &[u8]) -> io::Result<Option<Vec<u8>>> {
    if let Some(value) = self.memtable.get(key) {
        return Ok(value.map(<[u8]>::to_vec));
    }
    for table in self.level0.iter().rev() {
        if let Some(value) = table.get(key)? {
            return Ok(value);
        }
    }
    Ok(None)
}
```

There's a cost: the old value is still on disk, taking up space, and so is the tombstone. Neither goes away until **compaction** merges the files and drops both. That's the next thing I'll build.

## What Rust taught me along the way

**Borrowing makes flush ordering explicit.** The memtable's iterator hands out slices that point into the memtable's own memory. While those slices exist, the borrow checker won't let you replace or drop the memtable. So the flush has to be written in the safe order: write the file from the memtable, *then* replace it. A failed write leaves the data in memory, and the compiler enforces it.

**`&[u8]` over `&Vec<u8>`.** The SSTable writer accepts byte slices, not `Vec`s, so it works with any bytes: memtable contents, literals, or buffers read from other files (which compaction will need). Converting with `.as_slice()` is free; it just lends out the pointer and length.

## Where it stands, and what's next

Run `cargo run -p sstables` and the demo shows each piece working:

```
after crossing 32 bytes (expect 1 file): ["000001.sst"]
after waiting past 500ms (expect 2 files): ["000001.sst", "000002.sst"]
get apple (expect newer "red"): Some("red")
get cherry after delete (expect None, though 000001.sst still has it): None
simulated a crash mid-append: commit.log grew from 36 to 39 bytes
after reopening, commit.log is back to 36 bytes
get elderberry (expect "purple", replayed from the log): Some("purple")
```

What this engine still doesn't do, roughly in the order I plan to tackle it:

- **Every read scans whole files.** Bigtable splits each SSTable into 64 KB blocks with an index at the end of the file, so a lookup costs one disk seek.
- **Files pile up forever.** Compaction merges them, drops overwritten values, and finally removes tombstones.
- **Missing keys are expensive.** A lookup for a key that doesn't exist reads every file. **Bloom filters** let most of those skip the disk.
- **No checksums.** A corrupted byte in the log or a file wouldn't be detected.
- **Flushing blocks writes.** Bigtable freezes the full memtable and flushes it in the background while a new one takes writes.

## References

- Chang et al., [*Bigtable: A Distributed Storage System for Structured Data*](https://research.google.com/archive/bigtable-osdi06.pdf) (OSDI 2006), especially sections 4, 5.3, 5.4 and 6
- O'Neil et al., *The Log-Structured Merge-Tree (LSM-Tree)* (Acta Informatica, 1996)
- Alex Petrov, *Database Internals* (O'Reilly, 2019)
