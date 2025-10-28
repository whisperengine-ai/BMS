//! Vector search types and models

use bms_core::types::CoordId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata attached to vector embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMetadata {
    /// Coordinate ID
    pub coord_id: CoordId,
    
    /// Creation timestamp
    pub created_at: String,
    
    /// Optional author/source
    pub author: Option<String>,
    
    /// Optional tags for filtering
    pub tags: Vec<String>,
    
    /// Custom metadata fields
    pub custom: HashMap<String, serde_json::Value>,
}

impl VectorMetadata {
    pub fn new(coord_id: CoordId) -> Self {
        Self {
            coord_id,
            created_at: chrono::Utc::now().to_rfc3339(),
            author: None,
            tags: Vec::new(),
            custom: HashMap::new(),
        }
    }
    
    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }
    
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Query text to search for
    pub query: String,
    
    /// Maximum number of results
    #[serde(default = "default_limit")]
    pub limit: usize,
    
    /// Optional filters
    pub filter: Option<SearchFilter>,
    
    /// Minimum similarity score (0.0 - 1.0)
    #[serde(default)]
    pub min_score: Option<f32>,
}

fn default_limit() -> usize {
    10
}

/// Filter criteria for search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilter {
    /// Filter by author
    pub author: Option<String>,
    
    /// Filter by tags (any match)
    pub tags: Option<Vec<String>>,
    
    /// Filter by date range
    pub created_after: Option<String>,
    pub created_before: Option<String>,
}

/// Search result with score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Coordinate ID
    pub coord_id: CoordId,
    
    /// Similarity score (0.0 - 1.0)
    pub score: f32,
    
    /// Associated metadata
    pub metadata: VectorMetadata,
}

impl SearchResult {
    pub fn new(coord_id: CoordId, score: f32, metadata: VectorMetadata) -> Self {
        Self {
            coord_id,
            score,
            metadata,
        }
    }
}
