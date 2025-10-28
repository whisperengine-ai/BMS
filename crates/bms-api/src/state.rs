use bms_core::SnapshotManager;
use bms_storage::BmsRepository;
use bms_vector::VectorStore;

pub struct AppState {
    pub repository: BmsRepository,
    pub vector_store: VectorStore,
    pub snapshot_manager: SnapshotManager,
}
