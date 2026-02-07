//! Append-only data file for vector and payload storage.
//!
//! Vectors and their payloads are stored in a binary format for efficient
//! random access. Each record is addressed by its byte offset in the file.
//!
//! # Format
//!
//! Each record has the format:
//! ```text
//! [deleted:u8][id:u64][dim:u32][vector:f32*dim][payload_len:u32][payload:json]
//! ```

use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::payload::Payload;
use crate::vector::VectorId;

/// Marker for deleted records.
const DELETED_MARKER: u8 = 1;
const ACTIVE_MARKER: u8 = 0;

/// A record stored in the data file.
#[derive(Debug, Clone)]
pub struct DataRecord {
    /// Byte offset in the file.
    pub offset: u64,
    /// Whether the record is deleted.
    pub deleted: bool,
    /// Vector ID.
    pub id: VectorId,
    /// Vector data.
    pub vector: Vec<f32>,
    /// Metadata payload.
    pub payload: Payload,
}

/// Append-only data file for vector storage.
pub struct DataFile {
    /// Path to the data file.
    path: PathBuf,
    /// File handle for writing.
    writer: BufWriter<File>,
    /// Current write position (end of file).
    write_pos: u64,
}

impl DataFile {
    /// Opens or creates a data file.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&path)
            .map_err(|e| Error::IoError(format!("failed to open data file: {}", e)))?;

        let write_pos = file
            .metadata()
            .map_err(|e| Error::IoError(format!("metadata failed: {}", e)))?
            .len();

        Ok(Self {
            path,
            writer: BufWriter::new(file),
            write_pos,
        })
    }

    /// Appends a record to the data file.
    ///
    /// Returns the byte offset of the written record.
    pub fn append(&mut self, id: VectorId, vector: &[f32], payload: &Payload) -> Result<u64> {
        let offset = self.write_pos;
        let dim = vector.len() as u32;
        let payload_json = serde_json::to_vec(payload)
            .map_err(|e| Error::IoError(format!("payload serialization failed: {}", e)))?;
        let payload_len = payload_json.len() as u32;

        // Write: [deleted:1][id:8][dim:4][vector:dim*4][payload_len:4][payload:payload_len]
        self.writer
            .write_all(&[ACTIVE_MARKER])
            .map_err(|e| Error::IoError(format!("write deleted marker failed: {}", e)))?;

        self.writer
            .write_all(&id.to_le_bytes())
            .map_err(|e| Error::IoError(format!("write id failed: {}", e)))?;

        self.writer
            .write_all(&dim.to_le_bytes())
            .map_err(|e| Error::IoError(format!("write dim failed: {}", e)))?;

        for &val in vector {
            self.writer
                .write_all(&val.to_le_bytes())
                .map_err(|e| Error::IoError(format!("write vector failed: {}", e)))?;
        }

        self.writer
            .write_all(&payload_len.to_le_bytes())
            .map_err(|e| Error::IoError(format!("write payload_len failed: {}", e)))?;

        self.writer
            .write_all(&payload_json)
            .map_err(|e| Error::IoError(format!("write payload failed: {}", e)))?;

        // Calculate record size: 1 + 8 + 4 + dim*4 + 4 + payload_len
        let record_size = 1 + 8 + 4 + (dim as u64 * 4) + 4 + payload_len as u64;
        self.write_pos += record_size;

        Ok(offset)
    }

    /// Marks a record as deleted at the given offset.
    pub fn mark_deleted(&self, offset: u64) -> Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .open(&self.path)
            .map_err(|e| Error::IoError(format!("open for delete failed: {}", e)))?;

        let mut writer = BufWriter::new(file);
        writer
            .seek(SeekFrom::Start(offset))
            .map_err(|e| Error::IoError(format!("seek failed: {}", e)))?;
        writer
            .write_all(&[DELETED_MARKER])
            .map_err(|e| Error::IoError(format!("write delete marker failed: {}", e)))?;
        writer
            .flush()
            .map_err(|e| Error::IoError(format!("flush failed: {}", e)))?;

        Ok(())
    }

    /// Reads a record at the given offset.
    pub fn read_at(&self, offset: u64) -> Result<DataRecord> {
        let file = File::open(&self.path)
            .map_err(|e| Error::IoError(format!("open for read failed: {}", e)))?;

        let mut reader = BufReader::new(file);
        reader
            .seek(SeekFrom::Start(offset))
            .map_err(|e| Error::IoError(format!("seek failed: {}", e)))?;

        // Read deleted marker
        let mut deleted_buf = [0u8; 1];
        reader
            .read_exact(&mut deleted_buf)
            .map_err(|e| Error::IoError(format!("read deleted failed: {}", e)))?;
        let deleted = deleted_buf[0] == DELETED_MARKER;

        // Read ID
        let mut id_buf = [0u8; 8];
        reader
            .read_exact(&mut id_buf)
            .map_err(|e| Error::IoError(format!("read id failed: {}", e)))?;
        let id = u64::from_le_bytes(id_buf);

        // Read dimension
        let mut dim_buf = [0u8; 4];
        reader
            .read_exact(&mut dim_buf)
            .map_err(|e| Error::IoError(format!("read dim failed: {}", e)))?;
        let dim = u32::from_le_bytes(dim_buf) as usize;

        // Read vector
        let mut vector = Vec::with_capacity(dim);
        for _ in 0..dim {
            let mut val_buf = [0u8; 4];
            reader
                .read_exact(&mut val_buf)
                .map_err(|e| Error::IoError(format!("read vector failed: {}", e)))?;
            vector.push(f32::from_le_bytes(val_buf));
        }

        // Read payload length
        let mut payload_len_buf = [0u8; 4];
        reader
            .read_exact(&mut payload_len_buf)
            .map_err(|e| Error::IoError(format!("read payload_len failed: {}", e)))?;
        let payload_len = u32::from_le_bytes(payload_len_buf) as usize;

        // Read payload
        let mut payload_buf = vec![0u8; payload_len];
        reader
            .read_exact(&mut payload_buf)
            .map_err(|e| Error::IoError(format!("read payload failed: {}", e)))?;
        let payload: Payload = serde_json::from_slice(&payload_buf)
            .map_err(|e| Error::IoError(format!("payload deserialize failed: {}", e)))?;

        Ok(DataRecord {
            offset,
            deleted,
            id,
            vector,
            payload,
        })
    }

    /// Iterates over all active (non-deleted) records.
    pub fn iter_active(&self) -> Result<Vec<DataRecord>> {
        let file = File::open(&self.path)
            .map_err(|e| Error::IoError(format!("open for iter failed: {}", e)))?;

        let file_len = file
            .metadata()
            .map_err(|e| Error::IoError(format!("metadata failed: {}", e)))?
            .len();

        if file_len == 0 {
            return Ok(Vec::new());
        }

        let mut records = Vec::new();
        let mut offset = 0u64;

        while offset < file_len {
            match self.read_at(offset) {
                Ok(record) => {
                    // Calculate record size
                    let record_size = 1
                        + 8
                        + 4
                        + (record.vector.len() as u64 * 4)
                        + 4
                        + serde_json::to_vec(&record.payload)
                            .unwrap_or_default()
                            .len() as u64;

                    if !record.deleted {
                        records.push(record);
                    }

                    offset += record_size;
                }
                Err(_) => break, // Reached end or corrupted data
            }
        }

        Ok(records)
    }

    /// Flushes pending writes to disk.
    pub fn flush(&mut self) -> Result<()> {
        self.writer
            .flush()
            .map_err(|e| Error::IoError(format!("flush failed: {}", e)))?;
        self.writer
            .get_ref()
            .sync_all()
            .map_err(|e| Error::IoError(format!("sync failed: {}", e)))?;
        Ok(())
    }

    /// Returns the current write position.
    pub fn write_position(&self) -> u64 {
        self.write_pos
    }

    /// Returns the path to the data file.
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

    fn temp_data_path() -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join("polarisdb_test_data");
        fs::create_dir_all(&dir).unwrap();
        dir.join(format!("test_{}_{}.pdb", std::process::id(), id))
    }

    #[test]
    fn test_data_file_append_and_read() {
        let path = temp_data_path();
        let _ = fs::remove_file(&path);

        let offset;
        {
            let mut df = DataFile::open(&path).unwrap();
            offset = df
                .append(
                    1,
                    &[1.0, 2.0, 3.0],
                    &Payload::new().with_field("key", "value"),
                )
                .unwrap();
            df.flush().unwrap();
        }

        let df = DataFile::open(&path).unwrap();
        let record = df.read_at(offset).unwrap();

        assert_eq!(record.id, 1);
        assert_eq!(record.vector, vec![1.0, 2.0, 3.0]);
        assert_eq!(record.payload.get_str("key"), Some("value"));
        assert!(!record.deleted);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_data_file_delete() {
        let path = temp_data_path();
        let _ = fs::remove_file(&path);

        let offset;
        {
            let mut df = DataFile::open(&path).unwrap();
            offset = df.append(1, &[1.0], &Payload::new()).unwrap();
            df.flush().unwrap();
        }

        {
            let df = DataFile::open(&path).unwrap();
            df.mark_deleted(offset).unwrap();
        }

        let df = DataFile::open(&path).unwrap();
        let record = df.read_at(offset).unwrap();
        assert!(record.deleted);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_data_file_iter_active() {
        let path = temp_data_path();
        let _ = fs::remove_file(&path);

        let offset1;
        {
            let mut df = DataFile::open(&path).unwrap();
            offset1 = df.append(1, &[1.0], &Payload::new()).unwrap();
            df.append(2, &[2.0], &Payload::new()).unwrap();
            df.append(3, &[3.0], &Payload::new()).unwrap();
            df.flush().unwrap();
        }

        // Delete first record
        {
            let df = DataFile::open(&path).unwrap();
            df.mark_deleted(offset1).unwrap();
        }

        let df = DataFile::open(&path).unwrap();
        let active = df.iter_active().unwrap();

        assert_eq!(active.len(), 2);
        assert_eq!(active[0].id, 2);
        assert_eq!(active[1].id, 3);

        fs::remove_file(&path).unwrap();
    }
}
