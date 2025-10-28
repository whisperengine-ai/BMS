//! BMS Vector - Vector search integration with Qdrant and FastEmbed
//!
//! Provides semantic search capabilities using:
//! - Qdrant in-memory mode for vector storage
//! - FastEmbed for generating embeddings

use bms_core::types::CoordId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod embedding;
mod memory_store;
mod types;

pub use embedding::EmbeddingGenerator;
pub use memory_store::InMemoryVectorStore;
pub use types::{SearchFilter, SearchQuery, SearchResult, VectorMetadata};

#[derive(Error, Debug)]
pub enum VectorError {
    #[error("Qdrant error: {0}")]
    Qdrant(#[from] anyhow::Error),
    
    #[error("Embedding error: {0}")]
    Embedding(String),
    
    #[error("Invalid vector dimension: expected {expected}, got {actual}")]
    InvalidDimension { expected: usize, actual: usize },
    
    #[error("Collection not found: {0}")]
    CollectionNotFound(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Vector store trait for different implementations
#[async_trait::async_trait]
pub trait VectorStore: Send + Sync {
    /// Store an embedding for a coordinate with metadata
    async fn store_embedding(
        &self,
        coord_id: &CoordId,
        embedding: Vec<f32>,
        metadata: VectorMetadata,
    ) -> Result<(), VectorError>;

    /// Search for similar coordinates by embedding vector
    async fn search_by_vector(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        filter: Option<SearchFilter>,
    ) -> Result<Vec<SearchResult>, VectorError>;

    /// Delete embedding for a coordinate
    async fn delete_embedding(&self, coord_id: &CoordId) -> Result<(), VectorError>;

    /// Get collection statistics
    async fn get_stats(&self) -> Result<VectorStats, VectorError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStats {
    pub total_vectors: u64,
    pub dimension: usize,
    pub indexed_vectors: u64,
}

/// Configuration for vector store
#[derive(Debug, Clone)]
pub struct VectorConfig {
    /// Path to store Qdrant data
    pub storage_path: String,
    
    /// Collection name
    pub collection_name: String,
    
    /// Vector dimension (384 for all-MiniLM-L6-v2)
    pub dimension: usize,
    
    /// HNSW index parameters
    pub hnsw_m: usize,
    pub hnsw_ef_construct: usize,
}

impl Default for VectorConfig {
    fn default() -> Self {
        Self {
            storage_path: "./qdrant_data".to_string(),
            collection_name: "bms_memory".to_string(),
            dimension: 384, // all-MiniLM-L6-v2 embedding size
            hnsw_m: 32,
            hnsw_ef_construct: 200,
        }
    }
}

/// Initialize vector store with embedding generator
pub fn init_vector_system(
    config: VectorConfig,
) -> Result<(Box<dyn VectorStore>, EmbeddingGenerator), VectorError> {
    // Initialize in-memory store
    let store = InMemoryVectorStore::new(config)?;
    
    // Initialize embedding generator
    let generator = EmbeddingGenerator::new()?;
    
    Ok((Box::new(store), generator))
}
