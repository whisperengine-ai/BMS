//! Embedding generation using FastEmbed

use crate::VectorError;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

/// Embedding generator using FastEmbed
pub struct EmbeddingGenerator {
    model: TextEmbedding,
    dimension: usize,
}

impl EmbeddingGenerator {
    /// Create a new embedding generator with default model (all-MiniLM-L6-v2)
    pub fn new() -> Result<Self, VectorError> {
        Self::with_model(EmbeddingModel::AllMiniLML6V2)
    }
    
    /// Create embedding generator with specific model
    pub fn with_model(model_type: EmbeddingModel) -> Result<Self, VectorError> {
        let options = InitOptions::new(model_type.clone());
        
        let model = TextEmbedding::try_new(options)
            .map_err(|e| VectorError::Embedding(format!("Failed to initialize model: {}", e)))?;
        
        let dimension = match model_type {
            EmbeddingModel::AllMiniLML6V2 => 384,
            EmbeddingModel::BGESmallENV15 => 384,
            EmbeddingModel::BGEBaseENV15 => 768,
            EmbeddingModel::BGELargeENV15 => 1024,
            _ => 384, // Default to 384
        };
        
        Ok(Self {
            model,
            dimension,
        })
    }
    
    /// Get the embedding dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }
    
    /// Generate embedding for a single text
        pub fn generate(&mut self, text: &str) -> Result<Vec<f32>, VectorError> {
        let mut embeddings = self.generate_batch(vec![text])?;
        embeddings.pop()
            .ok_or_else(|| VectorError::Embedding("No embedding generated".to_string()))
    }
    
    /// Generate embeddings for multiple texts (more efficient)
        pub fn generate_batch(&mut self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>, VectorError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        
        let texts_owned: Vec<String> = texts.into_iter().map(|s| s.to_string()).collect();
        let embeddings = self.model.embed(texts_owned, None)
        .map_err(|e| VectorError::Embedding(format!("Embedding generation failed: {}", e)))?;
        
        Ok(embeddings)
    }
    
    /// Generate embedding from JSON state (uses stringified JSON)
        pub fn generate_from_state(&mut self, state: &serde_json::Value) -> Result<Vec<f32>, VectorError> {
        let text = serde_json::to_string(state)
            .map_err(|e| VectorError::Embedding(format!("Failed to serialize state: {}", e)))?;
        
        self.generate(&text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_embedding() {
           let mut generator = EmbeddingGenerator::new().unwrap();
        
        let embedding = generator.generate("Hello world").unwrap();
        
        assert_eq!(embedding.len(), 384);
        assert!(embedding.iter().any(|&x| x != 0.0));
    }
    
    #[test]
    fn test_generate_batch() {
           let mut generator = EmbeddingGenerator::new().unwrap();
        
        let texts = vec!["First text", "Second text", "Third text"];
        let embeddings = generator.generate_batch(texts).unwrap();
        
        assert_eq!(embeddings.len(), 3);
        assert!(embeddings.iter().all(|e| e.len() == 384));
    }
    
    #[test]
    fn test_dimension() {
        let generator = EmbeddingGenerator::new().unwrap();
        assert_eq!(generator.dimension(), 384);
    }
}
