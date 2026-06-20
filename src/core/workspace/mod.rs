pub mod foundation;
pub mod legacy_state;
pub mod registry;
pub mod state_io;

// Re-export main public types and functions for convenience
pub use foundation::{
    get_workspace_changes_dir, get_workspace_metadata_dir, get_workspace_view_state_path,
    is_valid_workspace_link_name, is_valid_workspace_name, parse_workspace_view_state,
    serialize_workspace_view_state, validate_workspace_link_name, validate_workspace_name,
    write_file_atomically, ContextStoreBinding, ContextStoreSelector, OpenerKind,
    PreferredOpener, WorkspaceContext, WorkspaceInitiativeRef, WorkspaceSkillState,
    WorkspaceViewState, WORKSPACE_CHANGES_DIR_NAME, WORKSPACE_CODE_WORKSPACE_EXTENSION,
    WORKSPACE_METADATA_DIR_NAME, WORKSPACE_VIEW_STATE_FILE_NAME,
};

pub use registry::{
    get_managed_workspaces_dir, get_workspace_registry_path, load_workspace_registry,
    list_workspace_registry_entries, parse_workspace_registry_state,
    save_workspace_registry, serialize_workspace_registry_state, WorkspaceRegistryEntry,
    WorkspaceRegistryState, MANAGED_WORKSPACES_DIR_NAME, WORKSPACE_REGISTRY_FILE_NAME,
};

pub use legacy_state::{
    get_workspace_legacy_local_state_path, get_workspace_legacy_shared_state_path,
    parse_workspace_local_state, parse_workspace_shared_state, workspace_state_parts_to_view_state,
    WorkspaceLocalState, WorkspaceSharedState, WORKSPACE_LEGACY_LOCAL_STATE_FILE_NAME,
    WORKSPACE_LEGACY_SHARED_STATE_FILE_NAME,
};

pub use state_io::{
    find_workspace_root, is_workspace_root, read_optional_workspace_view_state,
    read_workspace_view_state, workspace_changes_dir_exists, write_workspace_view_state,
};
