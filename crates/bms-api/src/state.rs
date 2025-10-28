use bms_core::{CoordId, SnapshotManager};
use bms_storage::BmsRepository;
use bms_vector::EmbeddingGenerator;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Cached embedding for a coordinate head state
#[derive(Clone)]
pub struct CachedEmbedding {
    pub head_hash: String,
    pub embedding: Vec<f32>,
    pub author: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct AppState {
    pub repository: BmsRepository,
    /// In-memory cache of embeddings for coordinate heads (coord_id -> cached embedding)
    /// Design: vectors are search metadata, not canonical storage
    /// Embeddings are computed on-demand during search and cached by head hash
    pub embedding_cache: Arc<Mutex<HashMap<CoordId, CachedEmbedding>>>,
    pub embedding_generator: Mutex<EmbeddingGenerator>,
    pub snapshot_manager: SnapshotManager,
}
