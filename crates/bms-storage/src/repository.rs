use crate::models::{CoordRow, DeltaRow, SnapshotRow};
use crate::schema::SCHEMA_SQL;
use bms_core::types::{Coordinate, CoordId, Delta, DeltaId, Snapshot, SnapshotId};
use bms_core::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;
use tracing::info;

/// BMS repository for SQLite storage operations
pub struct BmsRepository {
    pool: SqlitePool,
}

impl BmsRepository {
    /// Create a new repository with the given database path
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let path_str = db_path.as_ref().to_str().ok_or_else(|| {
            bms_core::error::BmsError::Other("Invalid database path".to_string())
        })?;

        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path_str))?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let repo = Self { pool };
        repo.initialize_schema().await?;

        Ok(repo)
    }

    /// Initialize database schema
    async fn initialize_schema(&self) -> Result<()> {
        sqlx::query(SCHEMA_SQL).execute(&self.pool).await?;
        info!("Database schema initialized");
        Ok(())
    }

    /// Insert a new coordinate
    pub async fn insert_coordinate(&self, coord: &Coordinate) -> Result<()> {
        let metadata_json = coord
            .metadata
            .as_ref()
            .map(|m| serde_json::to_string(m))
            .transpose()?;

        sqlx::query(
            r#"
            INSERT INTO coordinates (id_ascii, rune_alias, created_at, metadata)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&coord.id.0)
        .bind(&coord.rune_alias)
        .bind(coord.created_at)
        .bind(metadata_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get a coordinate by ID
    pub async fn get_coordinate(&self, coord_id: &CoordId) -> Result<Option<Coordinate>> {
        let row: Option<CoordRow> = sqlx::query_as(
            r#"
            SELECT id_ascii, rune_alias, created_at, metadata
            FROM coordinates
            WHERE id_ascii = ?
            "#,
        )
        .bind(&coord_id.0)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    /// Check if coordinate exists
    pub async fn coordinate_exists(&self, coord_id: &CoordId) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM coordinates WHERE id_ascii = ?
            "#,
        )
        .bind(&coord_id.0)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    /// Insert a new delta
    pub async fn insert_delta(&self, delta: &Delta) -> Result<()> {
        let ops_json = serde_json::to_string(&delta.ops)?;
        let tags_json = delta
            .tags
            .as_ref()
            .map(|t| serde_json::to_string(t))
            .transpose()?;

        sqlx::query(
            r#"
            INSERT INTO deltas (
                id, coord_id, parent_id, parent_hash, delta_hash, chain_hash,
                ops, created_at, tags, author
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&delta.id.0)
        .bind(&delta.coord_id.0)
        .bind(delta.parent_id.as_ref().map(|id| &id.0))
        .bind(delta.parent_hash.as_ref().map(|h| &h.0))
        .bind(&delta.delta_hash.0)
        .bind(&delta.chain_hash.0)
        .bind(ops_json)
        .bind(delta.created_at)
        .bind(tags_json)
        .bind(&delta.author)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get deltas for a coordinate
    pub async fn get_deltas(&self, coord_id: &CoordId) -> Result<Vec<Delta>> {
        let rows: Vec<DeltaRow> = sqlx::query_as(
            r#"
            SELECT id, coord_id, parent_id, parent_hash, delta_hash, chain_hash,
                   ops, created_at, tags, author
            FROM deltas
            WHERE coord_id = ?
            ORDER BY created_at ASC
            "#,
        )
        .bind(&coord_id.0)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    /// Get delta by ID
    pub async fn get_delta(&self, delta_id: &DeltaId) -> Result<Option<Delta>> {
        let row: Option<DeltaRow> = sqlx::query_as(
            r#"
            SELECT id, coord_id, parent_id, parent_hash, delta_hash, chain_hash,
                   ops, created_at, tags, author
            FROM deltas
            WHERE id = ?
            "#,
        )
        .bind(&delta_id.0)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    /// Get delta count for a coordinate
    pub async fn get_delta_count(&self, coord_id: &CoordId) -> Result<u32> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM deltas WHERE coord_id = ?
            "#,
        )
        .bind(&coord_id.0)
        .fetch_one(&self.pool)
        .await?;

        Ok(count as u32)
    }

    /// Insert a snapshot
    pub async fn insert_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        let state_json = serde_json::to_string(&snapshot.state)?;

        sqlx::query(
            r#"
            INSERT INTO snapshots (id, coord_id, head_delta_id, state_hash, state, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&snapshot.id.0)
        .bind(&snapshot.coord_id.0)
        .bind(&snapshot.head_delta_id.0)
        .bind(&snapshot.state_hash.0)
        .bind(state_json)
        .bind(snapshot.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get latest snapshot for a coordinate
    pub async fn get_latest_snapshot(&self, coord_id: &CoordId) -> Result<Option<Snapshot>> {
        let row: Option<SnapshotRow> = sqlx::query_as(
            r#"
            SELECT id, coord_id, head_delta_id, state_hash, state, created_at
            FROM snapshots
            WHERE coord_id = ?
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(&coord_id.0)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    /// Get snapshot by ID
    pub async fn get_snapshot(&self, snapshot_id: &SnapshotId) -> Result<Option<Snapshot>> {
        let row: Option<SnapshotRow> = sqlx::query_as(
            r#"
            SELECT id, coord_id, head_delta_id, state_hash, state, created_at
            FROM snapshots
            WHERE id = ?
            "#,
        )
        .bind(&snapshot_id.0)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    /// Get all coordinates
    pub async fn list_coordinates(&self, limit: Option<i64>) -> Result<Vec<Coordinate>> {
        let limit = limit.unwrap_or(100);

        let rows: Vec<CoordRow> = sqlx::query_as(
            r#"
            SELECT id_ascii, rune_alias, created_at, metadata
            FROM coordinates
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> Result<StorageStats> {
        let coord_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM coordinates")
            .fetch_one(&self.pool)
            .await?;

        let delta_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM deltas")
            .fetch_one(&self.pool)
            .await?;

        let snapshot_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM snapshots")
            .fetch_one(&self.pool)
            .await?;

        Ok(StorageStats {
            coordinate_count: coord_count as u64,
            delta_count: delta_count as u64,
            snapshot_count: snapshot_count as u64,
        })
    }
}

#[derive(Debug, Clone)]
pub struct StorageStats {
    pub coordinate_count: u64,
    pub delta_count: u64,
    pub snapshot_count: u64,
}
