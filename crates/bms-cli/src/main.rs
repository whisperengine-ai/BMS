use anyhow::Result;
use bms_core::{types::*, CoordinateGenerator, DeltaEngine, SnapshotManager};
use bms_storage::BmsRepository;
use clap::{Parser, Subcommand};
use serde_json::Value;
use tracing::info;
use bms_vector::{EmbeddingGenerator, InMemoryVectorStore, VectorConfig, VectorMetadata, SearchFilter as VecSearchFilter, VectorStore};

#[derive(Parser)]
#[command(name = "bms")]
#[command(about = "Babel Memory System CLI", long_about = None)]
struct Cli {
    /// Database path
    #[arg(short, long, default_value = "./bms.db")]
    db_path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Store a new state
    Store {
        /// JSON state to store
        #[arg(short, long)]
        state: String,

        /// Optional coordinate hint
        #[arg(short, long)]
        coord: Option<String>,
    },

    /// Recall a state
    Recall {
        /// Coordinate ID
        coord_id: String,
    },

    /// List all coordinates
    List,

    /// Verify chain integrity
    Verify {
        /// Coordinate ID
        coord_id: String,
    },

    /// Show statistics
    Stats,

    /// Initialize database
    Init,

    /// Semantic search
    Search {
        /// Query text
        query: String,
        /// Max results
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
        /// Minimum score filter
        #[arg(long)]
        min_score: Option<f32>,
        /// Author filter
        #[arg(long)]
        author: Option<String>,
        /// Tags filter (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    let repo = BmsRepository::new(&cli.db_path).await?;
    info!("Connected to database: {}", cli.db_path);

    match cli.command {
        Commands::Store { state, coord } => {
            let state_value: Value = serde_json::from_str(&state)?;

            let coord_id = if let Some(hint) = coord {
                CoordId(hint)
            } else {
                CoordinateGenerator::generate_now(&state_value)?
            };

            // Create coordinate if new
            if !repo.coordinate_exists(&coord_id).await? {
                let coordinate = Coordinate {
                    id: coord_id.clone(),
                    rune_alias: None,
                    created_at: chrono::Utc::now(),
                    metadata: None,
                };
                repo.insert_coordinate(&coordinate).await?;
                println!("Created coordinate: {}", coord_id);
            }

            // Get deltas and compute new delta
            let deltas = repo.get_deltas(&coord_id).await?;
            let prev_state = if deltas.is_empty() {
                serde_json::json!({})
            } else {
                let mut state = serde_json::json!({});
                for delta in &deltas {
                    DeltaEngine::apply_delta(&mut state, &delta.ops)?;
                }
                state
            };

            let ops = DeltaEngine::compute_delta(&prev_state, &state_value)?;
            let delta_hash = DeltaEngine::hash_delta(&ops)?;
            let delta_id = DeltaEngine::generate_delta_id(&ops)?;

            let (parent_id, parent_hash) = if let Some(last) = deltas.last() {
                (Some(last.id.clone()), Some(last.chain_hash.clone()))
            } else {
                (None, None)
            };

            let chain_hash = if let Some(ref ph) = parent_hash {
                bms_core::MerkleChain::compute_chain_hash(ph, &delta_hash)
            } else {
                delta_hash.clone()
            };

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
                author: None,
            };

            repo.insert_delta(&delta).await?;

            println!("Stored delta: {}", delta_id);
            println!("Coordinate: {}", coord_id);
        }

        Commands::Recall { coord_id } => {
            let coord_id = CoordId(coord_id);
            let deltas = repo.get_deltas(&coord_id).await?;

            if deltas.is_empty() {
                println!("No deltas found for coordinate: {}", coord_id);
                return Ok(());
            }

            let mut state = serde_json::json!({});
            for delta in &deltas {
                DeltaEngine::apply_delta(&mut state, &delta.ops)?;
            }

            println!("State for {}:", coord_id);
            println!("{}", serde_json::to_string_pretty(&state)?);
            println!("\nDelta count: {}", deltas.len());
        }

        Commands::List => {
            let coords = repo.list_coordinates(None).await?;

            println!("Coordinates ({}):", coords.len());
            for coord in coords {
                println!("  {} (created: {})", coord.id, coord.created_at);
            }
        }

        Commands::Verify { coord_id } => {
            let coord_id = CoordId(coord_id);
            let deltas = repo.get_deltas(&coord_id).await?;

            let (verified, error) = bms_core::MerkleChain::verify_chain_integrity(&deltas);

            println!("Chain verification for {}:", coord_id);
            println!("  Total deltas: {}", deltas.len());
            println!("  Verified: {}", verified);

            if let Some(e) = error {
                println!("  Error: {}", e);
            } else {
                println!("  Status: ✓ Valid");
            }
        }

        Commands::Stats => {
            let stats = repo.get_stats().await?;

            println!("BMS Statistics:");
            println!("  Coordinates: {}", stats.coordinate_count);
            println!("  Deltas: {}", stats.delta_count);
            println!("  Snapshots: {}", stats.snapshot_count);
        }

        Commands::Init => {
            println!("Database initialized at: {}", cli.db_path);
        }

        Commands::Search { query, limit, min_score, author, tags } => {
            // If API URL is provided, call API; else local fallback
            if let Ok(api_url) = std::env::var("BMS_API_URL") {
                let url = format!("{}/search", api_url.trim_end_matches('/'));
                let client = reqwest::Client::new();
                let tags_vec = tags.as_ref().map(|s| s.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect::<Vec<_>>() );
                let body = serde_json::json!({
                    "query": query,
                    "limit": limit,
                    "min_score": min_score,
                    "author": author,
                    "tags": tags_vec,
                });
                let resp = client.post(url).json(&body).send().await?;
                if !resp.status().is_success() {
                    anyhow::bail!("API error: {}", resp.text().await.unwrap_or_default());
                }
                let json: serde_json::Value = resp.json().await?;
                println!("Search results:\n{}", serde_json::to_string_pretty(&json)?);
                return Ok(());
            }

            // Local fallback: build in-memory index from current heads
            info!("Building in-memory index from current data (no API URL set)...");
            let coords = repo.list_coordinates(None).await?;
            let mut generator = EmbeddingGenerator::new().map_err(|e| anyhow::anyhow!("Embedding init error: {}", e))?;
            let store = InMemoryVectorStore::new(VectorConfig::default())
                .map_err(|e| anyhow::anyhow!("Vector store init error: {}", e))?;

            for coord in &coords {
                // Reconstruct head state
                let deltas = repo.get_deltas(&coord.id).await?;
                if deltas.is_empty() { continue; }
                let state = if let Some(snapshot) = repo.get_latest_snapshot(&coord.id).await? {
                    SnapshotManager::reconstruct(&snapshot, &deltas[..])?
                } else {
                    let mut s = serde_json::json!({});
                    for d in &deltas { DeltaEngine::apply_delta(&mut s, &d.ops)?; }
                    s
                };
                // Embed and store
                let embedding = generator.generate_from_state(&state)
                    .map_err(|e| anyhow::anyhow!("Embedding error: {}", e))?;
                let metadata = VectorMetadata::new(coord.id.clone())
                    .with_author("unknown".to_string());
                store.store_embedding(&coord.id, embedding, metadata).await
                    .map_err(|e| anyhow::anyhow!("Vector store error: {}", e))?;
            }

            // Query embedding and search
            let q_embed = generator.generate(&query)
                .map_err(|e| anyhow::anyhow!("Embedding error: {}", e))?;
            let filter = if author.is_some() || tags.is_some() {
                Some(VecSearchFilter { author, tags: tags.map(|s| s.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect()), created_after: None, created_before: None })
            } else { None };
            let mut results = store.search_by_vector(q_embed, limit, filter).await
                .map_err(|e| anyhow::anyhow!("Search error: {}", e))?;
            if let Some(min) = min_score { results.retain(|r| r.score >= min); }
            println!("Top {} results:", results.len());
            for r in results { println!("  {}  (score: {:.4})", r.coord_id, r.score); }
        }
    }

    Ok(())
}
