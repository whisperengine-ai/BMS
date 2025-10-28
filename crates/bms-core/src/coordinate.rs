use crate::canonical::Canonicalizer;
use crate::error::{BmsError, Result};
use crate::types::CoordId;
use crate::COORD_ID_BYTES;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sha3::{Digest, Sha3_256};

/// Coordinate generator for telic addressing
///
/// Generates deterministic 128-bit coordinates from state + timestamp
/// using SHA3-256 and base32 encoding (ASCII canonical)
pub struct CoordinateGenerator;

impl CoordinateGenerator {
    /// Generate a coordinate ID from state and timestamp
    ///
    /// Algorithm:
    /// 1. Canonicalize state to deterministic bytes
    /// 2. Concatenate with ISO-8601 UTC timestamp
    /// 3. Hash with SHA3-256
    /// 4. Take first 16 bytes (128-bit)
    /// 5. Encode as base32 (no padding)
    pub fn generate(state: &Value, timestamp: &DateTime<Utc>) -> Result<CoordId> {
        let canonical_state = Canonicalizer::canonicalize(state)?;
        let timestamp_str = timestamp.to_rfc3339();
        
        // Concatenate: canonical_state + "|" + timestamp
        let mut input = canonical_state;
        input.push(b'|');
        input.extend_from_slice(timestamp_str.as_bytes());

        // Hash with SHA3-256
        let mut hasher = Sha3_256::new();
        hasher.update(&input);
        let hash = hasher.finalize();

        // Take first 16 bytes (128-bit)
        let seed = &hash[..COORD_ID_BYTES];

        // Encode as base32 (uppercase, no padding)
        let coord_id = base32::encode(base32::Alphabet::Rfc4648 { padding: false }, seed);

        Ok(CoordId(coord_id))
    }

    /// Generate with current UTC timestamp
    pub fn generate_now(state: &Value) -> Result<CoordId> {
        Self::generate(state, &Utc::now())
    }

    /// Validate coordinate ID format
    pub fn validate(coord_id: &str) -> Result<()> {
        // Base32 RFC 4648 without padding: A-Z, 2-7
        // 128 bits = 16 bytes = 26 base32 characters (ceiling of 128/5)
        if coord_id.len() != 26 {
            return Err(BmsError::InvalidCoordinate(format!(
                "Expected 26 characters, got {}",
                coord_id.len()
            )));
        }

        if !coord_id.chars().all(|c| c.is_ascii_uppercase() || ('2'..='7').contains(&c)) {
            return Err(BmsError::InvalidCoordinate(
                "Invalid base32 characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Generate with explicit nonce for collision resolution
    pub fn generate_with_nonce(
        state: &Value,
        timestamp: &DateTime<Utc>,
        nonce: u32,
    ) -> Result<CoordId> {
        let canonical_state = Canonicalizer::canonicalize(state)?;
        let timestamp_str = timestamp.to_rfc3339();
        
        let mut input = canonical_state;
        input.push(b'|');
        input.extend_from_slice(timestamp_str.as_bytes());
        input.push(b'|');
        input.extend_from_slice(&nonce.to_le_bytes());

        let mut hasher = Sha3_256::new();
        hasher.update(&input);
        let hash = hasher.finalize();

        let seed = &hash[..COORD_ID_BYTES];
        let coord_id = base32::encode(base32::Alphabet::Rfc4648 { padding: false }, seed);

        Ok(CoordId(coord_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;

    #[test]
    fn test_generate_coordinate() {
        let state = json!({"key": "value", "number": 42});
        let timestamp = Utc.with_ymd_and_hms(2025, 10, 28, 12, 0, 0).unwrap();

        let coord = CoordinateGenerator::generate(&state, &timestamp).unwrap();
        
        // Should be 26 characters (128 bits in base32)
        assert_eq!(coord.0.len(), 26);
        
        // Should be valid base32
        assert!(CoordinateGenerator::validate(&coord.0).is_ok());
    }

    #[test]
    fn test_deterministic_generation() {
        let state = json!({"a": 1, "b": 2});
        let timestamp = Utc.with_ymd_and_hms(2025, 10, 28, 12, 0, 0).unwrap();

        let coord1 = CoordinateGenerator::generate(&state, &timestamp).unwrap();
        let coord2 = CoordinateGenerator::generate(&state, &timestamp).unwrap();

        assert_eq!(coord1, coord2);
    }

    #[test]
    fn test_different_states_different_coords() {
        let state1 = json!({"key": "value1"});
        let state2 = json!({"key": "value2"});
        let timestamp = Utc.with_ymd_and_hms(2025, 10, 28, 12, 0, 0).unwrap();

        let coord1 = CoordinateGenerator::generate(&state1, &timestamp).unwrap();
        let coord2 = CoordinateGenerator::generate(&state2, &timestamp).unwrap();

        assert_ne!(coord1, coord2);
    }

    #[test]
    fn test_nonce_collision_resolution() {
        let state = json!({"key": "value"});
        let timestamp = Utc.with_ymd_and_hms(2025, 10, 28, 12, 0, 0).unwrap();

        let coord0 = CoordinateGenerator::generate(&state, &timestamp).unwrap();
        let coord1 = CoordinateGenerator::generate_with_nonce(&state, &timestamp, 1).unwrap();
        let coord2 = CoordinateGenerator::generate_with_nonce(&state, &timestamp, 2).unwrap();

        // All should be different
        assert_ne!(coord0, coord1);
        assert_ne!(coord1, coord2);
        assert_ne!(coord0, coord2);
    }

    #[test]
    fn test_validate_invalid_length() {
        let result = CoordinateGenerator::validate("TOOSHORT");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_chars() {
        let result = CoordinateGenerator::validate("ABCDEFGH12345678901234!!!!");
        assert!(result.is_err());
    }
}
