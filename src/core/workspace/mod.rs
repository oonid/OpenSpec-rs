pub mod foundation;
pub mod legacy_state;
pub mod openers;
pub mod open_surface;
pub mod registry;
pub mod state_io;

// Re-export main public types and functions for convenience
pub use foundation::{
    get_workspace_changes_dir, get_workspace_code_workspace_file_name, get_workspace_code_workspace_path,
    get_workspace_context_initiative_id, get_workspace_metadata_dir, get_workspace_view_state_path,
    is_valid_workspace_link_name, is_valid_workspace_name, parse_workspace_view_state,
    serialize_workspace_view_state, validate_workspace_link_name, validate_workspace_name,
    write_file_atomically, parse_workspace_preferred_opener_value, validate_workspace_preferred_opener,
    ContextStoreBinding, ContextStoreSelector, OpenerKind,
    PreferredOpener, WorkspaceContext, WorkspaceInitiativeRef, WorkspaceSkillState,
    WorkspaceViewState, WORKSPACE_CHANGES_DIR_NAME, WORKSPACE_CODE_WORKSPACE_EXTENSION,
    WORKSPACE_METADATA_DIR_NAME, WORKSPACE_VIEW_STATE_FILE_NAME,
    WORKSPACE_SUPPORTED_OPENER_VALUES, WORKSPACE_AGENT_OPENER_IDS, WORKSPACE_EDITOR_OPENER_IDS,
};

pub use openers::{
    get_default_workspace_opener_choice_value, get_workspace_opener_executable,
    get_workspace_opener_label, is_workspace_executable_available, list_workspace_opener_choices,
    WorkspaceOpenerChoice,
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

pub use open_surface::{
    apply_workspace_guidance_block, build_workspace_code_workspace_content,
    build_workspace_guidance_block, resolve_workspace_open_links, sync_workspace_open_surface,
    ResolvedContextStoreRef, ResolvedInitiativeRef, WorkspaceOpenLink,
    WorkspaceOpenResolvedContext, WorkspaceOpenSurfaceGeneration, WorkspaceOpenSurfaceLinks,
    WorkspaceSkippedOpenLink, WorkspaceSkippedReason, WORKSPACE_GUIDANCE_BODY,
    WORKSPACE_GUIDANCE_END_MARKER, WORKSPACE_GUIDANCE_START_MARKER,
    WORKSPACE_OPEN_INITIATIVE_FOLDER_LABEL, WORKSPACE_OPEN_ROOT_FOLDER_LABEL,
};
