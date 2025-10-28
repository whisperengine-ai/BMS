use crate::canonical::Canonicalizer;
use crate::error::{BmsError, Result};
use crate::types::{DeltaId, Hash};
use serde_json::Value;
use sha3::{Digest, Sha3_256};

/// Delta engine for RFC 6902 JSON Patch compression
pub struct DeltaEngine;

impl DeltaEngine {
    /// Compute delta from previous state to current state
    ///
    /// Returns JSON Patch operations (RFC 6902)
    pub fn compute_delta(
        prev_state: &Value,
        current_state: &Value,
    ) -> Result<Vec<json_patch::PatchOperation>> {
        let patch = json_patch::diff(prev_state, current_state);
        Ok(patch.0)
    }

    /// Apply delta to a state
    pub fn apply_delta(
        state: &mut Value,
        ops: &[json_patch::PatchOperation],
    ) -> Result<()> {
        let patch = json_patch::Patch(ops.to_vec());
        json_patch::patch(state, &patch)?;
        Ok(())
    }

    /// Compute hash of delta operations
    pub fn hash_delta(ops: &[json_patch::PatchOperation]) -> Result<Hash> {
        let delta_value = serde_json::to_value(ops)?;
        let canonical = Canonicalizer::canonicalize(&delta_value)?;
        
        let mut hasher = Sha3_256::new();
        hasher.update(&canonical);
        let hash = hasher.finalize();
        
        Ok(Hash(hex::encode(hash)))
    }

    /// Generate delta ID from hash (first 16 bytes)
    pub fn generate_delta_id(ops: &[json_patch::PatchOperation]) -> Result<DeltaId> {
        let delta_value = serde_json::to_value(ops)?;
        let canonical = Canonicalizer::canonicalize(&delta_value)?;
        
        let mut hasher = Sha3_256::new();
        hasher.update(&canonical);
        let hash = hasher.finalize();
        
        // First 16 bytes as hex
        let id = hex::encode(&hash[..16]);
        Ok(DeltaId(id))
    }

    /// Compute hash of a state
    pub fn hash_state(state: &Value) -> Result<Hash> {
        let canonical = Canonicalizer::canonicalize(state)?;
        
        let mut hasher = Sha3_256::new();
        hasher.update(&canonical);
        let hash = hasher.finalize();
        
        Ok(Hash(hex::encode(hash)))
    }

    /// Verify delta hash matches expected
    pub fn verify_delta_hash(
        ops: &[json_patch::PatchOperation],
        expected_hash: &Hash,
    ) -> Result<()> {
        let actual_hash = Self::hash_delta(ops)?;
        
        if actual_hash.0 != expected_hash.0 {
            return Err(BmsError::HashMismatch {
                expected: expected_hash.0.clone(),
                actual: actual_hash.0,
            });
        }
        
        Ok(())
    }

    /// Calculate compression ratio
    pub fn compression_ratio(original: &Value, delta_ops: &[json_patch::PatchOperation]) -> f64 {
        let original_size = serde_json::to_string(original).unwrap_or_default().len();
        let delta_size = serde_json::to_string(delta_ops).unwrap_or_default().len();
        
        if original_size == 0 {
            return 0.0;
        }
        
        1.0 - (delta_size as f64 / original_size as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_compute_delta() {
        let prev = json!({"a": 1, "b": 2});
        let current = json!({"a": 1, "b": 3, "c": 4});

        let ops = DeltaEngine::compute_delta(&prev, &current).unwrap();
        
        // Should have operations for changing b and adding c
        assert!(!ops.is_empty());
    }

    #[test]
    fn test_apply_delta() {
        let prev = json!({"a": 1, "b": 2});
        let current = json!({"a": 1, "b": 3, "c": 4});

        let ops = DeltaEngine::compute_delta(&prev, &current).unwrap();
        
        let mut reconstructed = prev.clone();
        DeltaEngine::apply_delta(&mut reconstructed, &ops).unwrap();

        assert_eq!(reconstructed, current);
    }

    #[test]
    fn test_hash_delta_deterministic() {
        let ops = vec![
            json_patch::PatchOperation::Replace(json_patch::ReplaceOperation {
                path: jsonptr::Pointer::new(&[jsonptr::Token::from_encoded("b")]),
                value: json!(3),
            }),
        ];

        let hash1 = DeltaEngine::hash_delta(&ops).unwrap();
        let hash2 = DeltaEngine::hash_delta(&ops).unwrap();

        assert_eq!(hash1.0, hash2.0);
    }

    #[test]
    fn test_verify_delta_hash() {
        let ops = vec![
            json_patch::PatchOperation::Add(json_patch::AddOperation {
                path: jsonptr::Pointer::new(&[jsonptr::Token::from_encoded("c")]),
                value: json!(4),
            }),
        ];

        let hash = DeltaEngine::hash_delta(&ops).unwrap();
        
        assert!(DeltaEngine::verify_delta_hash(&ops, &hash).is_ok());
    }

    #[test]
    fn test_compression_ratio() {
        let original = json!({
            "field1": "a very long string that takes up space",
            "field2": "another long string",
            "field3": "yet another long string",
            "field4": 12345
        });
        
        let modified = json!({
            "field1": "a very long string that takes up space",
            "field2": "another long string",
            "field3": "yet another long string",
            "field4": 67890  // Only this changed
        });

        let ops = DeltaEngine::compute_delta(&original, &modified).unwrap();
        let ratio = DeltaEngine::compression_ratio(&original, &ops);

        // Delta should be significantly smaller than full object
        assert!(ratio > 0.5);
    }
}
