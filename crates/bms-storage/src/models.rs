use bms_core::types::{Coordinate, CoordId, Delta, DeltaId, Snapshot, SnapshotId};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;

/// Database model for coordinates
#[derive(Debug, Clone, FromRow)]
pub struct CoordRow {
    pub id_ascii: String,
    pub rune_alias: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<String>, // JSON string
}

impl From<CoordRow> for Coordinate {
    fn from(row: CoordRow) -> Self {
        let metadata = row
            .metadata
            .and_then(|s| serde_json::from_str(&s).ok());

        Coordinate {
            id: CoordId(row.id_ascii),
            rune_alias: row.rune_alias,
            created_at: row.created_at,
            metadata,
        }
    }
}

/// Database model for deltas
#[derive(Debug, Clone, FromRow)]
pub struct DeltaRow {
    pub id: String,
    pub coord_id: String,
    pub parent_id: Option<String>,
    pub parent_hash: Option<String>,
    pub delta_hash: String,
    pub chain_hash: String,
    pub ops: String, // JSON string
    pub created_at: DateTime<Utc>,
    pub tags: Option<String>,
    pub author: Option<String>,
}

impl TryFrom<DeltaRow> for Delta {
    type Error = bms_core::error::BmsError;

    fn try_from(row: DeltaRow) -> Result<Self, Self::Error> {
        let ops: Vec<json_patch::PatchOperation> = serde_json::from_str(&row.ops)?;
        let tags = row.tags.and_then(|s| serde_json::from_str(&s).ok());

        Ok(Delta {
            id: DeltaId(row.id),
            coord_id: CoordId(row.coord_id),
            parent_id: row.parent_id.map(DeltaId),
            parent_hash: row.parent_hash.map(bms_core::types::Hash),
            delta_hash: bms_core::types::Hash(row.delta_hash),
            chain_hash: bms_core::types::Hash(row.chain_hash),
            ops,
            created_at: row.created_at,
            tags,
            author: row.author,
        })
    }
}

/// Database model for snapshots
#[derive(Debug, Clone, FromRow)]
pub struct SnapshotRow {
    pub id: String,
    pub coord_id: String,
    pub head_delta_id: String,
    pub state_hash: String,
    pub state: String, // JSON string
    pub created_at: DateTime<Utc>,
}

impl TryFrom<SnapshotRow> for Snapshot {
    type Error = bms_core::error::BmsError;

    fn try_from(row: SnapshotRow) -> Result<Self, Self::Error> {
        let state: Value = serde_json::from_str(&row.state)?;

        Ok(Snapshot {
            id: SnapshotId(row.id),
            coord_id: CoordId(row.coord_id),
            head_delta_id: DeltaId(row.head_delta_id),
            state_hash: bms_core::types::Hash(row.state_hash),
            state,
            created_at: row.created_at,
        })
    }
}
