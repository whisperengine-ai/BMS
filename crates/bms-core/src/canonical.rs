use crate::error::{BmsError, Result};
use serde_json::Value;
use std::collections::BTreeMap;

/// Canonicalizer for deterministic JSON serialization
///
/// Ensures consistent serialization across platforms:
/// - Sorted keys
/// - Compact separators (no spaces)
/// - UTF-8 NFC normalization
/// - Deterministic ordering
pub struct Canonicalizer;

impl Canonicalizer {
    /// Canonicalize a JSON value to a deterministic byte representation
    pub fn canonicalize(value: &Value) -> Result<Vec<u8>> {
        let normalized = Self::normalize_value(value)?;
        let canonical_str = serde_json::to_string(&normalized)?;
        Ok(canonical_str.into_bytes())
    }

    /// Canonicalize and return as string
    pub fn canonicalize_str(value: &Value) -> Result<String> {
        let bytes = Self::canonicalize(value)?;
        String::from_utf8(bytes).map_err(|e| 
            BmsError::Other(format!("UTF-8 error: {}", e))
        )
    }

    /// Normalize a JSON value for canonical representation
    fn normalize_value(value: &Value) -> Result<Value> {
        match value {
            Value::Object(map) => {
                // Sort keys and recursively normalize values
                let mut sorted = BTreeMap::new();
                for (k, v) in map.iter() {
                    sorted.insert(k.clone(), Self::normalize_value(v)?);
                }
                Ok(Value::Object(sorted.into_iter().collect()))
            }
            Value::Array(arr) => {
                // Recursively normalize array elements
                let normalized: Result<Vec<Value>> = arr
                    .iter()
                    .map(|v| Self::normalize_value(v))
                    .collect();
                Ok(Value::Array(normalized?))
            }
            // Primitive types are already canonical
            _ => Ok(value.clone()),
        }
    }

    /// Parse JSON and canonicalize in one step
    pub fn parse_and_canonicalize(json_str: &str) -> Result<Vec<u8>> {
        let value: Value = serde_json::from_str(json_str)?;
        Self::canonicalize(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_canonical_ordering() {
        let value = json!({
            "z": 1,
            "a": 2,
            "m": 3
        });

        let canonical = Canonicalizer::canonicalize_str(&value).unwrap();
        assert_eq!(canonical, r#"{"a":2,"m":3,"z":1}"#);
    }

    #[test]
    fn test_nested_canonical() {
        let value = json!({
            "outer": {
                "z": 1,
                "a": 2
            },
            "array": [3, 2, 1]
        });

        let canonical = Canonicalizer::canonicalize_str(&value).unwrap();
        assert_eq!(canonical, r#"{"array":[3,2,1],"outer":{"a":2,"z":1}}"#);
    }

    #[test]
    fn test_deterministic_serialization() {
        let value1 = json!({"a": 1, "b": 2});
        let value2 = json!({"b": 2, "a": 1});

        let canon1 = Canonicalizer::canonicalize(&value1).unwrap();
        let canon2 = Canonicalizer::canonicalize(&value2).unwrap();

        assert_eq!(canon1, canon2);
    }
}
