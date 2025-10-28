use crate::error::{BmsError, Result};
use crate::types::{Delta, Hash};
use sha3::{Digest, Sha3_256};

/// Merkle chain for tamper-evident delta linking
pub struct MerkleChain;

impl MerkleChain {
    /// Compute chain hash: SHA3-256(parent_hash + current_delta_hash)
    pub fn compute_chain_hash(parent_hash: &Hash, delta_hash: &Hash) -> Hash {
        let mut hasher = Sha3_256::new();
        
        // Hash concatenation of parent and current
        hasher.update(parent_hash.0.as_bytes());
        hasher.update(delta_hash.0.as_bytes());
        
        let hash = hasher.finalize();
        Hash(hex::encode(hash))
    }

    /// Verify a single delta's Merkle link
    pub fn verify_delta(delta: &Delta) -> Result<()> {
        // If this is the first delta (no parent), verify only delta hash
        if delta.parent_id.is_none() {
            return Ok(());
        }

        let parent_hash = delta.parent_hash.as_ref().ok_or_else(|| {
            BmsError::MerkleChainBroken {
                delta_id: delta.id.0.clone(),
            }
        })?;

        // Compute expected chain hash
        let expected_chain_hash = Self::compute_chain_hash(parent_hash, &delta.delta_hash);

        // Verify it matches
        if expected_chain_hash.0 != delta.chain_hash.0 {
            return Err(BmsError::HashMismatch {
                expected: expected_chain_hash.0,
                actual: delta.chain_hash.0.clone(),
            });
        }

        Ok(())
    }

    /// Verify an entire chain of deltas
    pub fn verify_chain(deltas: &[Delta]) -> Result<()> {
        for delta in deltas {
            Self::verify_delta(delta)?;
        }
        Ok(())
    }

    /// Find the break point in a chain (for healing)
    ///
    /// Returns the index of the first broken delta, or None if chain is valid
    pub fn find_break_point(deltas: &[Delta]) -> Option<usize> {
        for (idx, delta) in deltas.iter().enumerate() {
            if Self::verify_delta(delta).is_err() {
                return Some(idx);
            }
        }
        None
    }

    /// Verify chain integrity and return verified length
    pub fn verify_chain_integrity(deltas: &[Delta]) -> (usize, Option<BmsError>) {
        for (idx, delta) in deltas.iter().enumerate() {
            if let Err(e) = Self::verify_delta(delta) {
                return (idx, Some(e));
            }
        }
        (deltas.len(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CoordId, DeltaId};
    use chrono::Utc;

    fn mock_delta(
        id: &str,
        coord_id: &str,
        parent_id: Option<&str>,
        parent_hash: Option<&str>,
        delta_hash: &str,
    ) -> Delta {
        let parent_hash_obj = parent_hash.map(|h| Hash(h.to_string()));
        let chain_hash = if let Some(ph) = &parent_hash_obj {
            MerkleChain::compute_chain_hash(ph, &Hash(delta_hash.to_string()))
        } else {
            Hash(delta_hash.to_string())
        };

        Delta {
            id: DeltaId(id.to_string()),
            coord_id: CoordId(coord_id.to_string()),
            parent_id: parent_id.map(|s| DeltaId(s.to_string())),
            parent_hash: parent_hash_obj,
            delta_hash: Hash(delta_hash.to_string()),
            chain_hash,
            ops: vec![],
            created_at: Utc::now(),
            tags: None,
            author: None,
        }
    }

    #[test]
    fn test_compute_chain_hash() {
        let parent = Hash("abc123".to_string());
        let current = Hash("def456".to_string());

        let chain_hash = MerkleChain::compute_chain_hash(&parent, &current);

        // Should produce a valid hex string
        assert_eq!(chain_hash.0.len(), 64); // SHA3-256 hex = 64 chars
    }

    #[test]
    fn test_verify_first_delta() {
        let delta = mock_delta("d1", "c1", None, None, "hash1");

        assert!(MerkleChain::verify_delta(&delta).is_ok());
    }

    #[test]
    fn test_verify_linked_delta() {
        let delta = mock_delta("d2", "c1", Some("d1"), Some("hash1"), "hash2");

        assert!(MerkleChain::verify_delta(&delta).is_ok());
    }

    #[test]
    fn test_verify_broken_chain() {
        let mut delta = mock_delta("d2", "c1", Some("d1"), Some("hash1"), "hash2");
        
        // Corrupt the chain hash
        delta.chain_hash = Hash("corrupted".to_string());

        assert!(MerkleChain::verify_delta(&delta).is_err());
    }

    #[test]
    fn test_verify_chain() {
        let delta1 = mock_delta("d1", "c1", None, None, "hash1");
        let delta2 = mock_delta("d2", "c1", Some("d1"), Some("hash1"), "hash2");
        let delta3 = mock_delta("d3", "c1", Some("d2"), Some(&delta2.chain_hash.0), "hash3");

        let deltas = vec![delta1, delta2, delta3];

        assert!(MerkleChain::verify_chain(&deltas).is_ok());
    }

    #[test]
    fn test_find_break_point() {
        let delta1 = mock_delta("d1", "c1", None, None, "hash1");
        let mut delta2 = mock_delta("d2", "c1", Some("d1"), Some("hash1"), "hash2");
        delta2.chain_hash = Hash("corrupted".to_string());
        let delta3 = mock_delta("d3", "c1", Some("d2"), Some(&delta2.delta_hash.0), "hash3");

        let deltas = vec![delta1, delta2, delta3];

        let break_point = MerkleChain::find_break_point(&deltas);
        assert_eq!(break_point, Some(1)); // Second delta is broken
    }
}
