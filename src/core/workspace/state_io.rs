use std::path::{Path, PathBuf};

use super::foundation::{
    get_workspace_changes_dir, get_workspace_metadata_dir, get_workspace_view_state_path,
    parse_workspace_view_state, serialize_workspace_view_state, WorkspaceViewState,
};
use super::legacy_state::{
    get_workspace_legacy_local_state_path, get_workspace_legacy_shared_state_path,
    parse_workspace_local_state, parse_workspace_shared_state, workspace_state_parts_to_view_state,
};

// Helper to check if a path is a file
fn path_is_file(path: &Path) -> bool {
    match std::fs::metadata(path) {
        Ok(metadata) => metadata.is_file(),
        Err(_) => false,
    }
}

// Helper to check if a path is a directory
fn path_is_dir(path: &Path) -> bool {
    match std::fs::metadata(path) {
        Ok(metadata) => metadata.is_dir(),
        Err(_) => false,
    }
}

/// Returns true if this path is a valid workspace root.
/// A workspace root must have either:
/// 1. A valid view.yaml file (modern), OR
/// 2. A valid workspace.yaml file that parses as WorkspaceSharedState (legacy)
///
/// IMPORTANT: A foreign workspace.yaml (e.g., from Dagster) that does NOT parse
/// as OpenSpec shared state is NOT considered a workspace root.
pub fn is_workspace_root(workspace_root: &Path) -> bool {
    let view_path = get_workspace_view_state_path(workspace_root);
    if path_is_file(&view_path) {
        return true;
    }

    let legacy_shared_path = get_workspace_legacy_shared_state_path(workspace_root);
    if !path_is_file(&legacy_shared_path) {
        return false;
    }

    // File exists; try to parse it as OpenSpec shared state
    match std::fs::read_to_string(&legacy_shared_path) {
        Ok(content) => parse_workspace_shared_state(&content).is_ok(),
        Err(_) => false,
    }
}

/// Find the workspace root by walking up from the given start path.
/// Returns the canonicalized root directory, or None if no workspace ancestor found.
pub fn find_workspace_root(start_path: &Path) -> Option<PathBuf> {
    // Canonicalize start path (use its dir if it's a file)
    let initial_dir = if start_path.is_file() {
        start_path.parent().map(|p| p.to_path_buf())
    } else {
        Some(start_path.to_path_buf())
    };

    let mut current_dir = initial_dir?;

    loop {
        if is_workspace_root(&current_dir) {
            return Some(current_dir);
        }

        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            // Reached filesystem root
            return None;
        }
    }
}

/// Read workspace view state from the given workspace root.
/// Tries view.yaml first; if not found, falls back to legacy shared + local files.
pub fn read_workspace_view_state(workspace_root: &Path) -> Result<WorkspaceViewState, String> {
    let view_path = get_workspace_view_state_path(workspace_root);

    if path_is_file(&view_path) {
        let content = std::fs::read_to_string(&view_path)
            .map_err(|e| format!("Failed to read view state file: {}", e))?;
        return parse_workspace_view_state(&content);
    }

    // Fall back to legacy shared state
    let legacy_shared_path = get_workspace_legacy_shared_state_path(workspace_root);
    let shared_content = std::fs::read_to_string(&legacy_shared_path)
        .map_err(|e| format!("Failed to read legacy shared state: {}", e))?;

    let shared_state = parse_workspace_shared_state(&shared_content)?;

    // Try to read optional local state
    let legacy_local_path = get_workspace_legacy_local_state_path(workspace_root);
    let local_state = match std::fs::read_to_string(&legacy_local_path) {
        Ok(content) => Some(parse_workspace_local_state(&content)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            return Err(format!("Failed to read legacy local state: {}", e));
        }
    };

    Ok(workspace_state_parts_to_view_state(
        shared_state,
        local_state,
    ))
}

/// Read workspace view state, returning None if neither view.yaml nor legacy files exist.
pub fn read_optional_workspace_view_state(
    workspace_root: &Path,
) -> Result<Option<WorkspaceViewState>, String> {
    let view_path = get_workspace_view_state_path(workspace_root);

    if path_is_file(&view_path) {
        let content = std::fs::read_to_string(&view_path)
            .map_err(|e| format!("Failed to read view state file: {}", e))?;
        return parse_workspace_view_state(&content).map(Some);
    }

    let legacy_shared_path = get_workspace_legacy_shared_state_path(workspace_root);
    if !path_is_file(&legacy_shared_path) {
        return Ok(None);
    }

    let shared_content = std::fs::read_to_string(&legacy_shared_path)
        .map_err(|e| format!("Failed to read legacy shared state: {}", e))?;

    let shared_state = parse_workspace_shared_state(&shared_content)?;

    let legacy_local_path = get_workspace_legacy_local_state_path(workspace_root);
    let local_state = match std::fs::read_to_string(&legacy_local_path) {
        Ok(content) => Some(parse_workspace_local_state(&content)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            return Err(format!("Failed to read legacy local state: {}", e));
        }
    };

    Ok(Some(workspace_state_parts_to_view_state(
        shared_state,
        local_state,
    )))
}

/// Write workspace view state to the workspace root.
/// Creates .openspec-workspace directory and writes view.yaml.
pub fn write_workspace_view_state(
    workspace_root: &Path,
    state: &WorkspaceViewState,
) -> Result<(), String> {
    let metadata_dir = get_workspace_metadata_dir(workspace_root);
    let view_path = get_workspace_view_state_path(workspace_root);
    let content = serialize_workspace_view_state(state)?;

    // Create metadata directory
    std::fs::create_dir_all(&metadata_dir)
        .map_err(|e| format!("Failed to create metadata directory: {}", e))?;

    // Use atomic write
    use super::foundation::write_file_atomically;
    write_file_atomically(&view_path, &content)
        .map_err(|e| format!("Failed to write view state file: {}", e))
}

/// Check if the changes directory exists for the workspace.
pub fn workspace_changes_dir_exists(workspace_root: &Path) -> bool {
    let changes_dir = get_workspace_changes_dir(workspace_root);
    path_is_dir(&changes_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_is_workspace_root_with_view_yaml() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let ws_root = tmpdir.path();

        // Create metadata dir and view.yaml
        let metadata_dir = get_workspace_metadata_dir(ws_root);
        std::fs::create_dir_all(&metadata_dir).expect("failed to create metadata dir");

        let view_yaml = r#"version: 1
name: test-ws
context: null
links: {}
"#;

        let view_path = get_workspace_view_state_path(ws_root);
        std::fs::write(&view_path, view_yaml).expect("failed to write view.yaml");

        assert!(is_workspace_root(ws_root));
    }

    #[test]
    fn test_is_workspace_root_with_legacy_shared_yaml() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let ws_root = tmpdir.path();

        // Create metadata dir and workspace.yaml
        let metadata_dir = get_workspace_metadata_dir(ws_root);
        std::fs::create_dir_all(&metadata_dir).expect("failed to create metadata dir");

        let shared_yaml = r#"version: 1
name: legacy-ws
context: null
links: {}
"#;

        let shared_path = get_workspace_legacy_shared_state_path(ws_root);
        std::fs::write(&shared_path, shared_yaml).expect("failed to write workspace.yaml");

        assert!(is_workspace_root(ws_root));
    }

    #[test]
    fn test_is_workspace_root_rejects_foreign_workspace_yaml() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let ws_root = tmpdir.path();

        // Create metadata dir and a foreign workspace.yaml (Dagster-like)
        let metadata_dir = get_workspace_metadata_dir(ws_root);
        std::fs::create_dir_all(&metadata_dir).expect("failed to create metadata dir");

        let foreign_yaml = r#"dagster:
  project_name: my_project
"#;

        let shared_path = get_workspace_legacy_shared_state_path(ws_root);
        std::fs::write(&shared_path, foreign_yaml).expect("failed to write workspace.yaml");

        // Should NOT be recognized as workspace root
        assert!(!is_workspace_root(ws_root));
    }

    #[test]
    fn test_find_workspace_root_from_subdir() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let ws_root = tmpdir.path();

        // Create workspace structure
        let metadata_dir = get_workspace_metadata_dir(ws_root);
        std::fs::create_dir_all(&metadata_dir).expect("failed to create metadata dir");

        let view_yaml = r#"version: 1
name: test-ws
context: null
links: {}
"#;
        let view_path = get_workspace_view_state_path(ws_root);
        std::fs::write(&view_path, view_yaml).expect("failed to write view.yaml");

        // Create a subdirectory
        let subdir = ws_root.join("subdir");
        std::fs::create_dir(&subdir).expect("failed to create subdir");

        // Find workspace root from subdir
        let found = find_workspace_root(&subdir).expect("failed to find workspace root");
        assert_eq!(found, ws_root);
    }

    #[test]
    fn test_find_workspace_root_from_file_in_workspace() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let ws_root = tmpdir.path();

        // Create workspace structure
        let metadata_dir = get_workspace_metadata_dir(ws_root);
        std::fs::create_dir_all(&metadata_dir).expect("failed to create metadata dir");

        let view_yaml = r#"version: 1
name: test-ws
context: null
links: {}
"#;
        let view_path = get_workspace_view_state_path(ws_root);
        std::fs::write(&view_path, view_yaml).expect("failed to write view.yaml");

        // Create a file in the workspace
        let file_path = ws_root.join("some_file.txt");
        std::fs::write(&file_path, "content").expect("failed to write file");

        // Find workspace root from the file
        let found = find_workspace_root(&file_path).expect("failed to find workspace root");
        assert_eq!(found, ws_root);
    }

    #[test]
    fn test_find_workspace_root_none_when_not_found() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let dir = tmpdir.path();

        let result = find_workspace_root(dir);
        assert!(result.is_none());
    }

    #[test]
    fn test_read_workspace_view_state_from_view_yaml() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let ws_root = tmpdir.path();

        // Create metadata dir and view.yaml
        let metadata_dir = get_workspace_metadata_dir(ws_root);
        std::fs::create_dir_all(&metadata_dir).expect("failed to create metadata dir");

        let view_yaml = r#"version: 1
name: test-ws
context: null
links:
  repo: /abs/path
"#;

        let view_path = get_workspace_view_state_path(ws_root);
        std::fs::write(&view_path, view_yaml).expect("failed to write view.yaml");

        let state =
            read_workspace_view_state(ws_root).expect("failed to read workspace view state");
        assert_eq!(state.name, "test-ws");
        assert!(state.context.is_none());
        assert_eq!(
            state.links.get("repo"),
            Some(&Some("/abs/path".to_string()))
        );
    }

    #[test]
    fn test_read_optional_workspace_view_state_returns_none() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let ws_root = tmpdir.path();

        // Create metadata dir but no state files
        let metadata_dir = get_workspace_metadata_dir(ws_root);
        std::fs::create_dir_all(&metadata_dir).expect("failed to create metadata dir");

        let result = read_optional_workspace_view_state(ws_root).expect("failed to read state");
        assert!(result.is_none());
    }

    #[test]
    fn test_write_and_read_workspace_view_state() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let ws_root = tmpdir.path();

        use std::collections::BTreeMap;
        let mut links = BTreeMap::new();
        links.insert("repo".to_string(), Some("/abs/path".to_string()));
        links.insert("docs".to_string(), None);

        let state = WorkspaceViewState {
            version: 1,
            name: "test-ws".to_string(),
            context: None,
            links,
            preferred_opener: None,
            tools: None,
            workspace_skills: None,
        };

        write_workspace_view_state(ws_root, &state).expect("failed to write state");

        let read_state = read_workspace_view_state(ws_root).expect("failed to read state");
        assert_eq!(read_state, state);
    }

    #[test]
    fn test_workspace_changes_dir_exists() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let ws_root = tmpdir.path();

        // Initially should not exist
        assert!(!workspace_changes_dir_exists(ws_root));

        // Create changes dir
        let changes_dir = get_workspace_changes_dir(ws_root);
        std::fs::create_dir(&changes_dir).expect("failed to create changes dir");

        // Now should exist
        assert!(workspace_changes_dir_exists(ws_root));
    }
}
