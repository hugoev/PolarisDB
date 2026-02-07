//! Storage module for persistent vector storage.
//!
//! This module provides:
//! - WAL (Write-Ahead Log) for crash-safe atomic operations
//! - Append-only data file for vector/payload storage
//! - Memory-mapped file helpers

pub mod data_file;
pub mod wal;

pub use data_file::DataFile;
pub use wal::{SyncMode, Wal, WalEntry, WalEntryKind};
