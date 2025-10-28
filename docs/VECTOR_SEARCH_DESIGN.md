# Vector Search Architecture

## Design Principle (from BMS_DESIGN.txt)

> **Section 5.3 Vector Store:**  
> "Store vectors for coord heads (and optionally select deltas).  
> **Do not store canonical chain data in Chroma; keep that in KV/object.**"

## Key Insight

**Vectors are search metadata, NOT canonical storage.**

The BMS design separates two concerns:

1. **Canonical Storage** (KV/Object Store)
   - Deltas (RFC 6902 JSON Patch)
   - Snapshots (full states)
   - Merkle chains (verification)
   - **Purpose**: Deterministic, verifiable, compressed memory
   - **Storage**: SQLite tables (`deltas`, `snapshots`, `coords`)

2. **Vector Index** (Ephemeral/Cached)
   - Embeddings of coordinate head states
   - **Purpose**: Semantic search accelerator
   - **Storage**: In-memory cache (POC), optional ChromaDB (production)

## POC Implementation (Phase 2)

### What We Do NOT Store

❌ Embeddings are NOT persisted during `POST /store`  
❌ No VectorStore writes to disk  
❌ No persistent vector database in POC  

### What We DO

✅ Generate embeddings **on-demand** during `POST /search`  
✅ Cache embeddings by coordinate head hash  
✅ Automatic cache invalidation (head hash mismatch)  
✅ Cosine similarity search in-memory  

### Architecture Flow

```
┌──────────────────────────────────────────────────────────┐
│                     POST /store                          │
├──────────────────────────────────────────────────────────┤
│ 1. Compute delta from prev_state → current_state        │
│ 2. Generate delta_hash (SHA3-256)                       │
│ 3. Link merkle chain (parent_hash → chain_hash)         │
│ 4. Persist to SQLite (deltas table)                     │
│ 5. Create snapshot if needed                            │
│ ✓ DONE - No vector operations                           │
└──────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────┐
│                    POST /search                          │
├──────────────────────────────────────────────────────────┤
│ 1. Generate query embedding                              │
│ 2. List all coordinates from SQLite                      │
│ 3. For each coordinate:                                  │
│    a. Reconstruct head state (snapshot + deltas)         │
│    b. Compute head_hash = SHA3-256(head_state)           │
│    c. Check cache[coord_id]:                             │
│       - Hit & head_hash match? → Reuse embedding         │
│       - Miss or mismatch? → Generate & cache             │
│ 4. Cosine similarity(query_emb, coord_embeddings)       │
│ 5. Filter by min_score                                   │
│ 6. Return top-k coord_ids + scores                       │
└──────────────────────────────────────────────────────────┘
```

### Benefits

1. **Design Purity**: "Telic act of choosing" stored in deltas, not embeddings
2. **Zero Storage Overhead**: No vector database files in POC
3. **Edge-Friendly**: Perfect for Raspberry Pi 5 (8GB RAM)
4. **Always Fresh**: Embeddings derived from current canonical state (no drift)
5. **Cache Efficiency**: Reuse embeddings across searches until state changes

### Cache Strategy

```rust
pub struct CachedEmbedding {
    pub head_hash: String,        // SHA3-256 of reconstructed head state
    pub embedding: Vec<f32>,       // FastEmbed (all-MiniLM-L6-v2, 384-dim)
    pub author: Option<String>,    // For filtering
    pub created_at: DateTime<Utc>, // For LRU eviction (future)
}

// Cache key: CoordId → CachedEmbedding
// Invalidation: head_hash mismatch on next search
```

### Performance Characteristics (POC)

- **Search latency**: Linear in number of coordinates (O(n))
- **Embedding generation**: ~20-50ms per coordinate (FastEmbed on CPU)
- **Cache hit rate**: High for stable coordinates
- **Memory usage**: ~1.5 KB per cached embedding (384 dims × 4 bytes)

**Example**: 1,000 coordinates = ~1.5 MB cached embeddings

### Production Upgrade Path (Future)

When scaling beyond ~10K coordinates:

1. **Option A: Persistent ChromaDB**
   ```rust
   #[cfg(feature = "chroma")]
   use bms_vector::ChromaVectorStore;
   
   let vector_store = ChromaVectorStore::new(config)?;
   ```

2. **Option B: HNSW Index**
   - Config: M=32, ef_construction=200, ef_search=128
   - Sub-linear search: O(log n)
   - Persistent index on disk

3. **Feature Flag**
   ```toml
   [features]
   default = ["in-memory"]
   chroma = ["chromadb-rs"]
   ```

### Design Validation

✅ **Section 5.3**: "Store vectors for coord heads" → Done (on-demand)  
✅ **Section 5.3**: "Do not store canonical chain data in Chroma" → Verified (deltas in SQLite only)  
✅ **Section 7.2**: "Semantic Search + Telic Coherence Rerank" → Stage 1 implemented (cosine), Stage 2 pending (coherence scoring)  
✅ **Section 13**: "BMS compresses the telic act of choosing" → Deltas are canonical, vectors are ephemeral  

## CLI Behavior

CLI search works identically:
1. No `BMS_API_URL` set → Build in-memory index from local SQLite
2. Reconstruct all coordinate heads
3. Generate embeddings on-the-fly
4. Cosine similarity search
5. Return top-k results

```bash
# Local search (no API)
bms --db-path demo.db search "Hello World" --limit 5 --min-score 0.2

# API search (if BMS_API_URL set)
export BMS_API_URL=http://localhost:3000
bms search "Hello World" --limit 5
```

## Testing

All tests pass with on-demand indexing:
- ✅ Core unit tests: 24/24
- ✅ Demo script: All steps successful
- ✅ Search examples: Returns correct results with scores

## Summary

**Original concern**: "I thought the original design was to save space and not store in vector storage?"

**Resolution**: Correct! We've now aligned with the design:
- **Before**: Embeddings persisted during `/store` (wrong)
- **After**: Embeddings computed on-demand during `/search` (correct)
- **Result**: Zero persistent vector storage, full design compliance

---

*Reference: BMS_DESIGN.txt Section 5.3, authored by DD01_Buz*
