//! Simple in-memory vector store implementation
//!
//! This is a basic implementation for Phase 2. Can be enhanced with Qdrant later.

use crate::types::{SearchFilter, SearchResult, VectorMetadata};
use crate::{VectorConfig, VectorError, VectorStats, VectorStore};
use bms_core::types::CoordId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
struct VectorEntry {
    embedding: Vec<f32>,
    metadata: VectorMetadata,
}

/// Simple in-memory vector store
pub struct InMemoryVectorStore {
    vectors: Arc<RwLock<HashMap<String, VectorEntry>>>,
    dimension: usize,
}

impl InMemoryVectorStore {
    /// Create new in-memory vector store
    pub fn new(config: VectorConfig) -> Result<Self, VectorError> {
        Ok(Self {
            vectors: Arc::new(RwLock::new(HashMap::new())),
            dimension: config.dimension,
        })
    }
    
    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return 0.0;
        }
        
        dot_product / (magnitude_a * magnitude_b)
    }
    
    /// Apply filter to metadata
    fn matches_filter(metadata: &VectorMetadata, filter: &SearchFilter) -> bool {
        if let Some(author) = &filter.author {
            if metadata.author.as_ref() != Some(author) {
                return false;
            }
        }
        
        if let Some(required_tags) = &filter.tags {
            if !required_tags.iter().any(|tag| metadata.tags.contains(tag)) {
                return false;
            }
        }
        
        // TODO: Implement date filtering
        
        true
    }
}

#[async_trait::async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn store_embedding(
        &self,
        coord_id: &CoordId,
        embedding: Vec<f32>,
        metadata: VectorMetadata,
    ) -> Result<(), VectorError> {
        if embedding.len() != self.dimension {
            return Err(VectorError::InvalidDimension {
                expected: self.dimension,
                actual: embedding.len(),
            });
        }
        
        let entry = VectorEntry {
            embedding,
            metadata,
        };
        
        let mut vectors = self.vectors.write()
            .map_err(|e| VectorError::Embedding(format!("Lock error: {}", e)))?;
        
        vectors.insert(coord_id.to_string(), entry);
        
        Ok(())
    }
    
    async fn search_by_vector(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        filter: Option<SearchFilter>,
    ) -> Result<Vec<SearchResult>, VectorError> {
        if query_embedding.len() != self.dimension {
            return Err(VectorError::InvalidDimension {
                expected: self.dimension,
                actual: query_embedding.len(),
            });
        }
        
        let vectors = self.vectors.read()
            .map_err(|e| VectorError::Embedding(format!("Lock error: {}", e)))?;
        
        let mut results: Vec<_> = vectors
            .iter()
            .filter(|(_, entry)| {
                if let Some(ref f) = filter {
                    Self::matches_filter(&entry.metadata, f)
                } else {
                    true
                }
            })
            .map(|(coord_id, entry)| {
                let score = Self::cosine_similarity(&query_embedding, &entry.embedding);
                SearchResult::new(
                    CoordId::from(coord_id.clone()),
                    score,
                    entry.metadata.clone(),
                )
            })
            .collect();
        
        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top-k
        results.truncate(limit);
        
        Ok(results)
    }
    
    async fn delete_embedding(&self, coord_id: &CoordId) -> Result<(), VectorError> {
        let mut vectors = self.vectors.write()
            .map_err(|e| VectorError::Embedding(format!("Lock error: {}", e)))?;
        
        vectors.remove(&coord_id.to_string());
        
        Ok(())
    }
    
    async fn get_stats(&self) -> Result<VectorStats, VectorError> {
        let vectors = self.vectors.read()
            .map_err(|e| VectorError::Embedding(format!("Lock error: {}", e)))?;
        
        Ok(VectorStats {
            total_vectors: vectors.len() as u64,
            dimension: self.dimension,
            indexed_vectors: vectors.len() as u64,
        })
    }
}
