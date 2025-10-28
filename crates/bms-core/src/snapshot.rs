use crate::delta::DeltaEngine;
use crate::error::{BmsError, Result};
use crate::types::{CoordId, Delta, Snapshot, SnapshotId};
use serde_json::Value;

/// Snapshot manager for efficient state reconstruction
pub struct SnapshotManager {
    snapshot_interval: u32,
}

impl SnapshotManager {
    pub fn new(snapshot_interval: u32) -> Self {
        Self { snapshot_interval }
    }

    /// Check if a snapshot should be created based on delta count
    pub fn should_snapshot(&self, delta_count: u32) -> bool {
        delta_count % self.snapshot_interval == 0
    }

    /// Create a snapshot from current state
    pub fn create_snapshot(
        &self,
        coord_id: CoordId,
        head_delta_id: crate::types::DeltaId,
        state: Value,
    ) -> Result<Snapshot> {
        let state_hash = DeltaEngine::hash_state(&state)?;
        
        // Generate snapshot ID from state hash
        let snapshot_id = SnapshotId(state_hash.0[..32].to_string());

        Ok(Snapshot {
            id: snapshot_id,
            coord_id,
            head_delta_id,
            state_hash,
            state,
            created_at: chrono::Utc::now(),
        })
    }

    /// Reconstruct state from snapshot and forward deltas
    pub fn reconstruct(
        snapshot: &Snapshot,
        deltas: &[Delta],
    ) -> Result<Value> {
        let mut state = snapshot.state.clone();

        // Apply each delta in order
        for delta in deltas {
            DeltaEngine::apply_delta(&mut state, &delta.ops)?;
        }

        Ok(state)
    }

    /// Verify snapshot integrity
    pub fn verify_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        let computed_hash = DeltaEngine::hash_state(&snapshot.state)?;

        if computed_hash.0 != snapshot.state_hash.0 {
            return Err(BmsError::HashMismatch {
                expected: snapshot.state_hash.0.clone(),
                actual: computed_hash.0,
            });
        }

        Ok(())
    }

    /// Find nearest snapshot before or at target delta
    pub fn find_nearest_snapshot<'a>(
        snapshots: &'a [Snapshot],
        _target_delta_id: &crate::types::DeltaId,
    ) -> Option<&'a Snapshot> {
        // In practice, would use timestamps or delta ordering
        // For now, return the last snapshot
        snapshots.last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CoordId, DeltaId};
    use serde_json::json;

    #[test]
    fn test_should_snapshot() {
        let manager = SnapshotManager::new(10);

        assert!(!manager.should_snapshot(5));
        assert!(manager.should_snapshot(10));
        assert!(!manager.should_snapshot(11));
        assert!(manager.should_snapshot(20));
    }

    #[test]
    fn test_create_snapshot() {
        let manager = SnapshotManager::new(10);
        let state = json!({"key": "value", "number": 42});

        let snapshot = manager
            .create_snapshot(
                CoordId("test_coord".to_string()),
                DeltaId("test_delta".to_string()),
                state.clone(),
            )
            .unwrap();

        assert_eq!(snapshot.coord_id.0, "test_coord");
        assert_eq!(snapshot.state, state);
    }

    #[test]
    fn test_verify_snapshot() {
        let manager = SnapshotManager::new(10);
        let state = json!({"key": "value"});

        let snapshot = manager
            .create_snapshot(
                CoordId("test".to_string()),
                DeltaId("delta".to_string()),
                state,
            )
            .unwrap();

        assert!(manager.verify_snapshot(&snapshot).is_ok());
    }

    #[test]
    fn test_reconstruct_from_snapshot() {
        let manager = SnapshotManager::new(10);
        let initial_state = json!({"a": 1, "b": 2});

        let snapshot = manager
            .create_snapshot(
                CoordId("test".to_string()),
                DeltaId("d1".to_string()),
                initial_state.clone(),
            )
            .unwrap();

        // Create a delta that modifies the state
        let new_state = json!({"a": 1, "b": 3, "c": 4});
        let ops = DeltaEngine::compute_delta(&initial_state, &new_state).unwrap();
        let delta_hash = DeltaEngine::hash_delta(&ops).unwrap();

        let delta = Delta {
            id: DeltaId("d2".to_string()),
            coord_id: CoordId("test".to_string()),
            parent_id: Some(DeltaId("d1".to_string())),
            parent_hash: Some(snapshot.state_hash.clone()),
            delta_hash: delta_hash.clone(),
            chain_hash: delta_hash,
            ops,
            created_at: chrono::Utc::now(),
            tags: None,
            author: None,
        };

        let reconstructed = SnapshotManager::reconstruct(&snapshot, &[delta]).unwrap();

        assert_eq!(reconstructed, new_state);
    }
}
