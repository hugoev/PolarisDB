//! Write-Ahead Log (WAL) for crash-safe atomic operations.
//!
//! The WAL ensures durability by writing all operations to disk before
//! applying them to the in-memory index. On crash recovery, the WAL is
//! replayed to restore the index to its last consistent state.
//!
//! # Format
//!
//! Each WAL entry has the format:
//! ```text
//! [checksum:u32][length:u32][kind:u8][id:u64][data...]
//! ```

use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::payload::Payload;
use crate::vector::VectorId;

/// Sync mode for WAL writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyncMode {
    /// Sync after every write (safest, slowest).
    Immediate,
    /// Sync after a batch of writes.
    #[default]
    Batched,
    /// Don't sync (fastest, risk of data loss on crash).
    NoSync,
}

/// The kind of operation in a WAL entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum WalEntryKind {
    /// Insert a new vector.
    Insert = 1,
    /// Update an existing vector.
    Update = 2,
    /// Delete a vector.
    Delete = 3,
    /// Checkpoint marker (WAL can be truncated before this point).
    Checkpoint = 4,
}

impl TryFrom<u8> for WalEntryKind {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::Insert),
            2 => Ok(Self::Update),
            3 => Ok(Self::Delete),
            4 => Ok(Self::Checkpoint),
            _ => Err(Error::WalCorrupted(format!(
                "invalid entry kind: {}",
                value
            ))),
        }
    }
}

/// A single WAL entry representing an operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    /// The kind of operation.
    pub kind: WalEntryKind,
    /// The vector ID (0 for checkpoint).
    pub id: VectorId,
    /// The vector data (empty for delete/checkpoint).
    pub vector: Vec<f32>,
    /// The payload (empty for delete/checkpoint).
    pub payload: Payload,
}

impl WalEntry {
    /// Creates an insert entry.
    pub fn insert(id: VectorId, vector: Vec<f32>, payload: Payload) -> Self {
        Self {
            kind: WalEntryKind::Insert,
            id,
            vector,
            payload,
        }
    }

    /// Creates an update entry.
    pub fn update(id: VectorId, vector: Vec<f32>, payload: Payload) -> Self {
        Self {
            kind: WalEntryKind::Update,
            id,
            vector,
            payload,
        }
    }

    /// Creates a delete entry.
    pub fn delete(id: VectorId) -> Self {
        Self {
            kind: WalEntryKind::Delete,
            id,
            vector: Vec::new(),
            payload: Payload::new(),
        }
    }

    /// Creates a checkpoint entry.
    pub fn checkpoint() -> Self {
        Self {
            kind: WalEntryKind::Checkpoint,
            id: 0,
            vector: Vec::new(),
            payload: Payload::new(),
        }
    }

    /// Serializes the entry to bytes.
    fn to_bytes(&self) -> Result<Vec<u8>> {
        let json = serde_json::to_vec(self)
            .map_err(|e| Error::WalCorrupted(format!("serialization failed: {}", e)))?;
        Ok(json)
    }

    /// Deserializes an entry from bytes.
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes)
            .map_err(|e| Error::WalCorrupted(format!("deserialization failed: {}", e)))
    }
}

/// Write-Ahead Log for durable operations.
pub struct Wal {
    /// Path to the WAL file.
    path: PathBuf,
    /// File handle for writing.
    writer: BufWriter<File>,
    /// Sync mode.
    sync_mode: SyncMode,
    /// Number of entries since last sync.
    entries_since_sync: usize,
    /// Batch size for syncing.
    batch_size: usize,
}

impl Wal {
    /// Opens or creates a WAL file.
    pub fn open<P: AsRef<Path>>(path: P, sync_mode: SyncMode) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&path)
            .map_err(|e| Error::IoError(format!("failed to open WAL: {}", e)))?;

        Ok(Self {
            path,
            writer: BufWriter::new(file),
            sync_mode,
            entries_since_sync: 0,
            batch_size: 100,
        })
    }

    /// Appends an entry to the WAL.
    pub fn append(&mut self, entry: &WalEntry) -> Result<()> {
        let data = entry.to_bytes()?;
        let checksum = crc32fast::hash(&data);
        let length = data.len() as u32;

        // Write: [checksum:4][length:4][data:length]
        self.writer
            .write_all(&checksum.to_le_bytes())
            .map_err(|e| Error::IoError(format!("write checksum failed: {}", e)))?;
        self.writer
            .write_all(&length.to_le_bytes())
            .map_err(|e| Error::IoError(format!("write length failed: {}", e)))?;
        self.writer
            .write_all(&data)
            .map_err(|e| Error::IoError(format!("write data failed: {}", e)))?;

        self.entries_since_sync += 1;

        // Sync based on mode
        match self.sync_mode {
            SyncMode::Immediate => self.sync()?,
            SyncMode::Batched if self.entries_since_sync >= self.batch_size => self.sync()?,
            _ => {}
        }

        Ok(())
    }

    /// Forces a sync to disk.
    pub fn sync(&mut self) -> Result<()> {
        self.writer
            .flush()
            .map_err(|e| Error::IoError(format!("flush failed: {}", e)))?;
        self.writer
            .get_ref()
            .sync_all()
            .map_err(|e| Error::IoError(format!("sync failed: {}", e)))?;
        self.entries_since_sync = 0;
        Ok(())
    }

    /// Writes a checkpoint and truncates the WAL.
    pub fn checkpoint(&mut self) -> Result<()> {
        self.append(&WalEntry::checkpoint())?;
        self.sync()?;

        // Close current writer, truncate file, and reopen
        drop(std::mem::replace(
            &mut self.writer,
            BufWriter::new(
                File::create(&self.path)
                    .map_err(|e| Error::IoError(format!("truncate failed: {}", e)))?,
            ),
        ));

        Ok(())
    }

    /// Reads all entries from the WAL for recovery.
    pub fn read_all<P: AsRef<Path>>(path: P) -> Result<Vec<WalEntry>> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(path)
            .map_err(|e| Error::IoError(format!("failed to open WAL for read: {}", e)))?;

        let file_len = file
            .metadata()
            .map_err(|e| Error::IoError(format!("metadata failed: {}", e)))?
            .len();

        if file_len == 0 {
            return Ok(Vec::new());
        }

        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();

        loop {
            // Read checksum
            let mut checksum_buf = [0u8; 4];
            match reader.read_exact(&mut checksum_buf) {
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(Error::IoError(format!("read checksum failed: {}", e))),
            }
            let expected_checksum = u32::from_le_bytes(checksum_buf);

            // Read length
            let mut length_buf = [0u8; 4];
            reader
                .read_exact(&mut length_buf)
                .map_err(|e| Error::IoError(format!("read length failed: {}", e)))?;
            let length = u32::from_le_bytes(length_buf) as usize;

            // Read data
            let mut data = vec![0u8; length];
            reader
                .read_exact(&mut data)
                .map_err(|e| Error::IoError(format!("read data failed: {}", e)))?;

            // Verify checksum
            let actual_checksum = crc32fast::hash(&data);
            if actual_checksum != expected_checksum {
                return Err(Error::WalCorrupted(format!(
                    "checksum mismatch: expected {}, got {}",
                    expected_checksum, actual_checksum
                )));
            }

            // Deserialize entry
            let entry = WalEntry::from_bytes(&data)?;
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Returns the path to the WAL file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_wal_path() -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join("polarisdb_test_wal");
        fs::create_dir_all(&dir).unwrap();
        dir.join(format!("test_{}_{}.wal", std::process::id(), id))
    }

    #[test]
    fn test_wal_append_and_read() {
        let path = temp_wal_path();
        let _ = fs::remove_file(&path);

        // Write entries
        {
            let mut wal = Wal::open(&path, SyncMode::Immediate).unwrap();
            wal.append(&WalEntry::insert(1, vec![1.0, 2.0, 3.0], Payload::new()))
                .unwrap();
            wal.append(&WalEntry::insert(
                2,
                vec![4.0, 5.0, 6.0],
                Payload::new().with_field("key", "value"),
            ))
            .unwrap();
            wal.append(&WalEntry::delete(1)).unwrap();
        }

        // Read entries
        let entries = Wal::read_all(&path).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].kind, WalEntryKind::Insert);
        assert_eq!(entries[0].id, 1);
        assert_eq!(entries[1].id, 2);
        assert_eq!(entries[2].kind, WalEntryKind::Delete);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_wal_checkpoint() {
        let path = temp_wal_path();
        let _ = fs::remove_file(&path);

        // Write and checkpoint
        {
            let mut wal = Wal::open(&path, SyncMode::Immediate).unwrap();
            wal.append(&WalEntry::insert(1, vec![1.0], Payload::new()))
                .unwrap();
            wal.checkpoint().unwrap();
        }

        // After checkpoint, WAL should be empty
        let entries = Wal::read_all(&path).unwrap();
        assert!(entries.is_empty());

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_wal_empty_file() {
        let path = temp_wal_path();
        let _ = fs::remove_file(&path);
        File::create(&path).unwrap();

        let entries = Wal::read_all(&path).unwrap();
        assert!(entries.is_empty());

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_wal_entry_serialization() {
        let entry = WalEntry::insert(
            42,
            vec![1.0, 2.0, 3.0],
            Payload::new().with_field("test", "value"),
        );

        let bytes = entry.to_bytes().unwrap();
        let recovered = WalEntry::from_bytes(&bytes).unwrap();

        assert_eq!(recovered.kind, WalEntryKind::Insert);
        assert_eq!(recovered.id, 42);
        assert_eq!(recovered.vector, vec![1.0, 2.0, 3.0]);
        assert_eq!(recovered.payload.get_str("test"), Some("value"));
    }
}
