//! BMS Core - Babel Memory System Core Library
//!
//! This crate implements the fundamental primitives of the BMS:
//! - Canonical JSON serialization
//! - Coordinate generation (telic addressing)
//! - Delta compression (RFC 6902 JSON Patch)
//! - Merkle chain verification
//! - Snapshot management

pub mod canonical;
pub mod coordinate;
pub mod delta;
pub mod error;
pub mod merkle;
pub mod snapshot;
pub mod types;

pub use canonical::Canonicalizer;
pub use coordinate::CoordinateGenerator;
pub use delta::DeltaEngine;
pub use error::{BmsError, Result};
pub use merkle::MerkleChain;
pub use snapshot::SnapshotManager;
pub use types::*;

/// BMS version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default snapshot interval (number of deltas before forced snapshot)
pub const DEFAULT_SNAPSHOT_INTERVAL: u32 = 128;

/// Coordinate ID length in bytes (128-bit)
pub const COORD_ID_BYTES: usize = 16;

/// Hash output length (SHA3-256)
pub const HASH_BYTES: usize = 32;
