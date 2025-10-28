use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Coordinate ID (ASCII base32, 128-bit deterministic address)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CoordId(pub String);

impl CoordId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for CoordId {
    fn from(s: String) -> Self {
        CoordId(s)
    }
}

impl std::fmt::Display for CoordId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Delta ID (SHA3-256 hash of delta, first 16 bytes hex)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeltaId(pub String);

impl DeltaId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for DeltaId {
    fn from(s: String) -> Self {
        DeltaId(s)
    }
}

impl std::fmt::Display for DeltaId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Snapshot ID (SHA3-256 hash of state, first 16 bytes hex)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotId(pub String);

impl SnapshotId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SnapshotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Hash value (SHA3-256, 32 bytes)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hash(pub String);

impl Hash {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Coordinate metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinate {
    pub id: CoordId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rune_alias: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Delta (JSON Patch with Merkle linking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    pub id: DeltaId,
    pub coord_id: CoordId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<DeltaId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_hash: Option<Hash>,
    pub delta_hash: Hash,
    pub chain_hash: Hash,
    pub ops: Vec<json_patch::PatchOperation>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

/// Snapshot (full state at a point in the delta chain)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: SnapshotId,
    pub coord_id: CoordId,
    pub head_delta_id: DeltaId,
    pub state_hash: Hash,
    pub state: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Compression statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    pub original_bytes: usize,
    pub compressed_bytes: usize,
    pub compression_ratio: f64,
    pub delta_count: u32,
}

impl CompressionStats {
    pub fn new(original_bytes: usize, compressed_bytes: usize, delta_count: u32) -> Self {
        let compression_ratio = if original_bytes > 0 {
            1.0 - (compressed_bytes as f64 / original_bytes as f64)
        } else {
            0.0
        };

        Self {
            original_bytes,
            compressed_bytes,
            compression_ratio,
            delta_count,
        }
    }
}
