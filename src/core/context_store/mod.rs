pub mod foundation;
pub mod operations;
pub mod registry;

// Re-export main public types and functions for convenience
pub use foundation::{
    get_context_store_metadata_dir, get_context_store_metadata_path,
    get_context_store_registry_path, get_context_stores_dir, get_default_context_store_root,
    is_valid_context_store_id, parse_metadata_state, parse_registry_state,
    serialize_metadata_state, serialize_registry_state, validate_context_store_id,
    write_file_atomically, BackendConfig, ContextStoreRegistryEntry, MetadataState,
    RegistryEntryState, RegistryState, CONTEXT_STORES_DIR_NAME, CONTEXT_STORE_METADATA_DIR_NAME,
    CONTEXT_STORE_METADATA_FILE_NAME, CONTEXT_STORE_REGISTRY_FILE_NAME,
};

pub use registry::{
    assert_no_registered_store_conflict, get_store_root_for_backend, list_registry_entries,
    load_registry, save_registry,
};

pub use operations::{
    doctor_context_stores, list_context_stores, register_existing_context_store,
    remove_context_store, setup_context_store, unregister_context_store, CleanupResult,
    ContextStoreInfo, DoctorResult, GitStatus, ListResult, MutationResult, StoreInspection,
};
