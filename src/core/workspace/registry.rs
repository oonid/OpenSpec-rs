use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::core::config::xdg_data_dir;

use super::foundation::validate_workspace_name;

// Constants
pub const MANAGED_WORKSPACES_DIR_NAME: &str = "workspaces";
pub const WORKSPACE_REGISTRY_FILE_NAME: &str = "registry.yaml";

// Registry state structures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceRegistryState {
    pub version: u8,
    pub workspaces: BTreeMap<String, String>, // name -> workspace_root
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceRegistryEntry {
    pub name: String,
    pub workspace_root: String,
}

// Path helpers
pub fn get_managed_workspaces_dir(global_data_dir: Option<&Path>) -> PathBuf {
    let base = match global_data_dir {
        Some(dir) => dir.to_path_buf(),
        None => xdg_data_dir(),
    };
    base.join(MANAGED_WORKSPACES_DIR_NAME)
}

pub fn get_workspace_registry_path(global_data_dir: Option<&Path>) -> PathBuf {
    get_managed_workspaces_dir(global_data_dir).join(WORKSPACE_REGISTRY_FILE_NAME)
}

// Parse/serialize helpers
pub fn parse_workspace_registry_state(content: &str) -> Result<WorkspaceRegistryState, String> {
    let parsed: WorkspaceRegistryState = serde_yaml::from_str(content)
        .map_err(|e| format!("Failed to parse workspace registry state: {}", e))?;

    // Validate all workspace names
    for name in parsed.workspaces.keys() {
        validate_workspace_name(name)
            .map_err(|e| format!("Invalid workspace registry name '{}': {}", name, e))?;
    }

    Ok(parsed)
}

pub fn serialize_workspace_registry_state(
    state: &WorkspaceRegistryState,
) -> Result<String, String> {
    // Validate all workspace names
    for name in state.workspaces.keys() {
        validate_workspace_name(name)
            .map_err(|e| format!("Invalid workspace registry name '{}': {}", name, e))?;
    }

    serde_yaml::to_string(state)
        .map_err(|e| format!("Failed to serialize workspace registry state: {}", e))
}

pub fn list_workspace_registry_entries(
    state: &WorkspaceRegistryState,
) -> Vec<WorkspaceRegistryEntry> {
    let mut entries: Vec<WorkspaceRegistryEntry> = state
        .workspaces
        .iter()
        .map(|(name, workspace_root)| WorkspaceRegistryEntry {
            name: name.clone(),
            workspace_root: workspace_root.clone(),
        })
        .collect();

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

pub fn load_workspace_registry(
    global_data_dir: Option<&Path>,
) -> Result<WorkspaceRegistryState, String> {
    let registry_path = get_workspace_registry_path(global_data_dir);

    match std::fs::read_to_string(&registry_path) {
        Ok(content) => parse_workspace_registry_state(&content),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Return empty registry if not found
            Ok(WorkspaceRegistryState {
                version: 1,
                workspaces: BTreeMap::new(),
            })
        }
        Err(e) => Err(format!("Failed to read workspace registry: {}", e)),
    }
}

pub fn save_workspace_registry(
    state: &WorkspaceRegistryState,
    global_data_dir: Option<&Path>,
) -> Result<(), String> {
    let registry_path = get_workspace_registry_path(global_data_dir);
    let content = serialize_workspace_registry_state(state)?;

    // Ensure parent directory exists
    if let Some(parent) = registry_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create registry directory: {}", e))?;
    }

    // Use atomic write
    use super::foundation::write_file_atomically;
    write_file_atomically(&registry_path, &content)
        .map_err(|e| format!("Failed to write workspace registry: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_registry_round_trip() {
        let mut workspaces = BTreeMap::new();
        workspaces.insert("my-ws".to_string(), "/abs/path/to/ws".to_string());
        workspaces.insert("another-ws".to_string(), "/abs/path/to/another".to_string());

        let state = WorkspaceRegistryState {
            version: 1,
            workspaces,
        };

        let serialized = serialize_workspace_registry_state(&state).expect("serialize failed");
        assert!(serialized.contains("version: 1"));
        assert!(serialized.contains("my-ws: /abs/path/to/ws"));

        let parsed = parse_workspace_registry_state(&serialized).expect("parse failed");
        assert_eq!(parsed, state);
    }

    #[test]
    fn test_registry_list_sorted_by_name() {
        let mut workspaces = BTreeMap::new();
        workspaces.insert("z-workspace".to_string(), "/path/z".to_string());
        workspaces.insert("a-workspace".to_string(), "/path/a".to_string());
        workspaces.insert("m-workspace".to_string(), "/path/m".to_string());

        let state = WorkspaceRegistryState {
            version: 1,
            workspaces,
        };

        let entries = list_workspace_registry_entries(&state);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].name, "a-workspace");
        assert_eq!(entries[1].name, "m-workspace");
        assert_eq!(entries[2].name, "z-workspace");
    }

    #[test]
    fn test_registry_load_missing_returns_empty() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let result = load_workspace_registry(Some(tmpdir.path())).expect("load failed");
        assert_eq!(result.version, 1);
        assert!(result.workspaces.is_empty());
    }

    #[test]
    fn test_registry_save_and_load() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");

        let mut workspaces = BTreeMap::new();
        workspaces.insert("test-ws".to_string(), "/path/to/test".to_string());

        let state = WorkspaceRegistryState {
            version: 1,
            workspaces,
        };

        save_workspace_registry(&state, Some(tmpdir.path())).expect("save failed");

        let loaded = load_workspace_registry(Some(tmpdir.path())).expect("load failed");
        assert_eq!(loaded, state);
    }

    #[test]
    fn test_registry_invalid_name_rejected() {
        let mut workspaces = BTreeMap::new();
        workspaces.insert("InvalidName".to_string(), "/path".to_string());

        let state = WorkspaceRegistryState {
            version: 1,
            workspaces,
        };

        let result = serialize_workspace_registry_state(&state);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Invalid workspace registry name"));
    }
}
