use thiserror::Error;

pub type Result<T> = std::result::Result<T, BmsError>;

#[derive(Error, Debug)]
pub enum BmsError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid coordinate: {0}")]
    InvalidCoordinate(String),

    #[error("Delta compression failed: {0}")]
    DeltaCompression(String),

    #[error("Hash verification failed: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("Merkle chain broken at delta {delta_id}")]
    MerkleChainBroken { delta_id: String },

    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),

    #[error("Delta not found: {0}")]
    DeltaNotFound(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Reconstruction failed: {0}")]
    ReconstructionFailed(String),

    #[error("Collision detected for coordinate: {0}")]
    CoordinateCollision(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<json_patch::PatchError> for BmsError {
    fn from(err: json_patch::PatchError) -> Self {
        BmsError::DeltaCompression(err.to_string())
    }
}

impl From<anyhow::Error> for BmsError {
    fn from(err: anyhow::Error) -> Self {
        BmsError::Other(err.to_string())
    }
}

// Re-export for external crates (when sqlx feature is enabled)
#[cfg(feature = "sqlx-support")]
impl From<sqlx::Error> for BmsError {
    fn from(err: sqlx::Error) -> Self {
        BmsError::Other(format!("Database error: {}", err))
    }
}
