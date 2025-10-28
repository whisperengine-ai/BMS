use axum::{
    routing::{get, post},
    Router,
};
use bms_core::{SnapshotManager, DEFAULT_SNAPSHOT_INTERVAL};
use bms_storage::BmsRepository;
use bms_vector::EmbeddingGenerator;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use tracing::info;

mod handlers;
mod state;

pub use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("Starting BMS API server...");

    // Initialize storage
    let db_path = std::env::var("BMS_DB_PATH").unwrap_or_else(|_| "./bms.db".to_string());
    let repository = BmsRepository::new(&db_path).await?;
    info!("Database initialized at {}", db_path);

    // Initialize embedding generator
    // Design note: vectors are search metadata, not canonical storage
    // Embeddings computed on-demand during search, cached in memory
    let embedding_generator = EmbeddingGenerator::new()
        .map_err(|e| anyhow::anyhow!("Failed to init embedding generator: {}", e))?;
    info!("Embedding generator initialized");

    // Initialize snapshot manager
    let snapshot_manager = SnapshotManager::new(DEFAULT_SNAPSHOT_INTERVAL);

    // Create shared state
    let state = Arc::new(AppState {
        repository,
        embedding_cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
        embedding_generator: tokio::sync::Mutex::new(embedding_generator),
        snapshot_manager,
    });

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/store", post(handlers::store_state))
        .route("/recall/:coord_id", get(handlers::recall_state))
        .route("/verify/:coord_id", get(handlers::verify_chain))
        .route("/snapshot/:coord_id", post(handlers::create_snapshot))
        .route("/coords", get(handlers::list_coordinates))
    .route("/stats", get(handlers::get_stats))
    .route("/search", post(handlers::search))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("BMS API listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> axum::response::Json<serde_json::Value> {
    axum::response::Json(serde_json::json!({
        "status": "ok",
        "version": bms_core::VERSION
    }))
}
