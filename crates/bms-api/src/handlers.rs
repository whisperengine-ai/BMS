use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use bms_core::{
    types::*, CoordinateGenerator, DeltaEngine, MerkleChain,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::state::AppState;

type ApiResult<T> = std::result::Result<T, AppError>;

#[derive(Debug, Deserialize)]
pub struct StoreRequest {
    pub coord_hint: Option<String>,
    pub state: serde_json::Value,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub author: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StoreResponse {
    pub coord_id: String,
    pub delta_id: String,
    pub snapshot_created: bool,
}

/// Store a new state
pub async fn store_state(
    State(app): State<Arc<AppState>>,
    Json(req): Json<StoreRequest>,
) -> ApiResult<Json<StoreResponse>> {
    info!("Storing new state");

    // Generate or retrieve coordinate
    let coord_id = if let Some(hint) = req.coord_hint {
        CoordId(hint)
    } else {
        CoordinateGenerator::generate_now(&req.state)?
    };

    // Check if coordinate exists, if not create it
    if !app.repository.coordinate_exists(&coord_id).await? {
        let coordinate = Coordinate {
            id: coord_id.clone(),
            rune_alias: None,
            created_at: chrono::Utc::now(),
            metadata: req.metadata,
        };
        app.repository.insert_coordinate(&coordinate).await?;
        info!("Created new coordinate: {}", coord_id);
    }

    // Get previous deltas
    let deltas = app.repository.get_deltas(&coord_id).await?;
    let delta_count = deltas.len() as u32;

    // Get previous state for delta computation
    let prev_state = if let Some(snapshot) = app.repository.get_latest_snapshot(&coord_id).await? {
        // Reconstruct from snapshot
        bms_core::SnapshotManager::reconstruct(&snapshot, &deltas[..])?
    } else if deltas.is_empty() {
        // First state for this coordinate
        serde_json::json!({})
    } else {
        // Reconstruct from all deltas
        let mut state = serde_json::json!({});
        for delta in &deltas {
            DeltaEngine::apply_delta(&mut state, &delta.ops)?;
        }
        state
    };

    // Compute delta
    let ops = DeltaEngine::compute_delta(&prev_state, &req.state)?;
    let delta_hash = DeltaEngine::hash_delta(&ops)?;
    let delta_id = DeltaEngine::generate_delta_id(&ops)?;

    // Get parent info
    let (parent_id, parent_hash) = if let Some(last_delta) = deltas.last() {
        (Some(last_delta.id.clone()), Some(last_delta.chain_hash.clone()))
    } else {
        (None, None)
    };

    // Compute chain hash
    let chain_hash = if let Some(ref ph) = parent_hash {
        MerkleChain::compute_chain_hash(ph, &delta_hash)
    } else {
        delta_hash.clone()
    };

    // Create delta
    let delta = Delta {
        id: delta_id.clone(),
        coord_id: coord_id.clone(),
        parent_id,
        parent_hash,
        delta_hash,
        chain_hash,
        ops,
        created_at: chrono::Utc::now(),
        tags: None,
        author: req.author,
    };

    // Store delta
    app.repository.insert_delta(&delta).await?;

    // Check if snapshot needed
    let mut snapshot_created = false;
    if app.snapshot_manager.should_snapshot(delta_count + 1) {
        let snapshot = app.snapshot_manager.create_snapshot(
            coord_id.clone(),
            delta_id.clone(),
            req.state.clone(),
        )?;
        app.repository.insert_snapshot(&snapshot).await?;
        snapshot_created = true;
        info!("Created snapshot for coordinate: {}", coord_id);
    }

    Ok(Json(StoreResponse {
        coord_id: coord_id.0,
        delta_id: delta_id.0,
        snapshot_created,
    }))
}

#[derive(Debug, Deserialize)]
pub struct RecallQuery {
    pub delta_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RecallResponse {
    pub coord_id: String,
    pub state: serde_json::Value,
    pub delta_count: u32,
}

/// Recall a state by coordinate ID
pub async fn recall_state(
    State(app): State<Arc<AppState>>,
    Path(coord_id_str): Path<String>,
    Query(query): Query<RecallQuery>,
) -> ApiResult<Json<RecallResponse>> {
    let coord_id = CoordId(coord_id_str);
    info!("Recalling state for coordinate: {}", coord_id);

    // Get all deltas
    let deltas = app.repository.get_deltas(&coord_id).await?;
    let delta_count = deltas.len() as u32;

    if deltas.is_empty() {
        return Err(AppError::NotFound(format!(
            "No deltas found for coordinate: {}",
            coord_id
        )));
    }

    // Get latest snapshot
    let state = if let Some(snapshot) = app.repository.get_latest_snapshot(&coord_id).await? {
        // Reconstruct from snapshot
        bms_core::SnapshotManager::reconstruct(&snapshot, &deltas[..])?
    } else {
        // Reconstruct from all deltas
        let mut state = serde_json::json!({});
        for delta in &deltas {
            DeltaEngine::apply_delta(&mut state, &delta.ops)?;
        }
        state
    };

    Ok(Json(RecallResponse {
        coord_id: coord_id.0,
        state,
        delta_count,
    }))
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub coord_id: String,
    pub verified_deltas: usize,
    pub total_deltas: usize,
    pub chain_valid: bool,
    pub first_break: Option<usize>,
}

/// Verify chain integrity
pub async fn verify_chain(
    State(app): State<Arc<AppState>>,
    Path(coord_id_str): Path<String>,
) -> ApiResult<Json<VerifyResponse>> {
    let coord_id = CoordId(coord_id_str);
    info!("Verifying chain for coordinate: {}", coord_id);

    let deltas = app.repository.get_deltas(&coord_id).await?;
    let total = deltas.len();

    let (verified, first_break) = MerkleChain::verify_chain_integrity(&deltas);

    Ok(Json(VerifyResponse {
        coord_id: coord_id.0,
        verified_deltas: verified,
        total_deltas: total,
        chain_valid: first_break.is_none(),
        first_break: if first_break.is_some() {
            Some(verified)
        } else {
            None
        },
    }))
}

/// Force create a snapshot
pub async fn create_snapshot(
    State(app): State<Arc<AppState>>,
    Path(coord_id_str): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let coord_id = CoordId(coord_id_str);
    info!("Creating snapshot for coordinate: {}", coord_id);

    // Reconstruct current state
    let deltas = app.repository.get_deltas(&coord_id).await?;
    if deltas.is_empty() {
        return Err(AppError::NotFound(format!(
            "No deltas found for coordinate: {}",
            coord_id
        )));
    }

    let state = if let Some(snapshot) = app.repository.get_latest_snapshot(&coord_id).await? {
        bms_core::SnapshotManager::reconstruct(&snapshot, &deltas[..])?
    } else {
        let mut state = serde_json::json!({});
        for delta in &deltas {
            DeltaEngine::apply_delta(&mut state, &delta.ops)?;
        }
        state
    };

    let head_delta_id = deltas.last().unwrap().id.clone();
    let snapshot = app
        .snapshot_manager
        .create_snapshot(coord_id, head_delta_id, state)?;

    app.repository.insert_snapshot(&snapshot).await?;

    Ok(Json(serde_json::json!({
        "snapshot_id": snapshot.id.0,
        "state_hash": snapshot.state_hash.0,
    })))
}

/// List coordinates
pub async fn list_coordinates(
    State(app): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<Coordinate>>> {
    let coords = app.repository.list_coordinates(Some(100)).await?;
    Ok(Json(coords))
}

/// Get storage statistics
pub async fn get_stats(
    State(app): State<Arc<AppState>>,
) -> ApiResult<Json<serde_json::Value>> {
    let stats = app.repository.get_stats().await?;

    Ok(Json(serde_json::json!({
        "coordinates": stats.coordinate_count,
        "deltas": stats.delta_count,
        "snapshots": stats.snapshot_count,
    })))
}

// Error handling
#[derive(Debug)]
pub enum AppError {
    BmsError(bms_core::error::BmsError),
    NotFound(String),
}

impl From<bms_core::error::BmsError> for AppError {
    fn from(err: bms_core::error::BmsError) -> Self {
        AppError::BmsError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::BmsError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        };

        let body = Json(serde_json::json!({
            "error": message
        }));

        (status, body).into_response()
    }
}
