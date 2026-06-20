use serde::Serialize;
use std::path::{Path, PathBuf};

use super::foundation::validate_workspace_link_name;
use super::state_io::{find_workspace_root, read_workspace_view_state};

/// A selected workspace, resolved either by name or by finding the current workspace root.
#[derive(Debug, Clone)]
pub struct SelectedWorkspace {
    pub name: String,
    pub root: PathBuf,
}

/// Resolve a workspace by name from the registry, or find one in the current directory tree.
/// If `workspace_name` is provided, look it up in the registry; otherwise find the workspace
/// containing `cwd`. Errors with a user-friendly message if neither succeeds.
pub fn resolve_selected_workspace(
    workspace_name: Option<&str>,
    cwd: &Path,
    gdd: Option<&Path>,
) -> Result<SelectedWorkspace, String> {
    if let Some(name) = workspace_name {
        // Look up workspace by name in the registry
        let registry = super::registry::load_workspace_registry(gdd)
            .map_err(|e| format!("Failed to load workspace registry: {}", e))?;

        match registry.workspaces.get(name) {
            Some(root_str) => {
                let root = PathBuf::from(root_str);
                Ok(SelectedWorkspace {
                    name: name.to_string(),
                    root,
                })
            }
            None => Err(format!("Unknown workspace '{}'.", name)),
        }
    } else {
        // Find workspace root from cwd
        match find_workspace_root(cwd) {
            Some(root) => {
                // Try to read the workspace name from its view state
                let name = match read_workspace_view_state(&root) {
                    Ok(view_state) => view_state.name,
                    Err(_) => {
                        // Fall back to directory basename
                        root.file_name()
                            .and_then(|n| n.to_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "workspace".to_string())
                    }
                };
                Ok(SelectedWorkspace { name, root })
            }
            None => Err("No workspace found. Run from inside a workspace or pass --workspace <name>.".to_string()),
        }
    }
}

/// Infer a link name from the last component of an absolute path.
pub fn infer_link_name(abs_path: &Path) -> String {
    abs_path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "link".to_string())
}

/// Resolve a path input to an absolute, canonicalized path that exists as a directory.
/// Returns errors for empty input, non-existent paths, or non-directories.
pub fn resolve_existing_directory(input: &str, cwd: &Path) -> Result<PathBuf, String> {
    if input.is_empty() {
        return Err("Repo or folder path must not be empty.".to_string());
    }

    let abs_path = if Path::new(input).is_absolute() {
        PathBuf::from(input)
    } else {
        cwd.join(input)
    };

    match std::fs::metadata(&abs_path) {
        Ok(metadata) => {
            if !metadata.is_dir() {
                return Err(format!(
                    "Path '{}' is not an existing folder.",
                    input
                ));
            }
            abs_path.canonicalize().map_err(|e| {
                format!("Failed to canonicalize path '{}': {}", input, e)
            })
        }
        Err(_) => Err(format!(
            "Path '{}' is not an existing folder.",
            input
        )),
    }
}

/// Add a workspace link to an existing workspace.
/// Returns (link_name, resolved_path_as_string).
pub fn add_workspace_link(
    selected: &SelectedWorkspace,
    name_or_path: &str,
    link_path: Option<&str>,
    cwd: &Path,
) -> Result<(String, String), String> {
    // Determine whether the first arg is a name or a path
    let explicit_name = if link_path.is_some() {
        Some(name_or_path)
    } else {
        None
    };
    let path_input = link_path.unwrap_or(name_or_path);

    // Resolve the directory
    let resolved = resolve_existing_directory(path_input, cwd)?;
    let resolved_str = resolved
        .to_str()
        .ok_or_else(|| "Path contains invalid UTF-8".to_string())?
        .to_string();

    // Determine and validate the link name
    let inferred_name = infer_link_name(&resolved);
    let link_name_str = explicit_name.unwrap_or(&inferred_name);
    validate_workspace_link_name(link_name_str)?;
    let link_name = link_name_str.to_string();

    // Read the current view state
    let mut view_state = super::state_io::read_workspace_view_state(&selected.root)?;

    // Check for duplicate link
    if view_state.links.contains_key(&link_name) {
        return Err(format!(
            "Workspace link '{}' already exists.",
            link_name
        ));
    }

    // Insert the link
    view_state.links.insert(link_name.clone(), Some(resolved_str.clone()));

    // Write the updated state
    super::state_io::write_workspace_view_state(&selected.root, &view_state)?;

    // Sync the open surface
    super::open_surface::sync_workspace_open_surface(&selected.root, &view_state, None)
        .map_err(|e| format!("Failed to sync workspace open surface: {}", e))?;

    Ok((link_name, resolved_str))
}

/// Update an existing workspace link.
/// Returns (link_name, resolved_path_as_string).
pub fn update_workspace_link(
    selected: &SelectedWorkspace,
    link_name_input: &str,
    link_path: &str,
    cwd: &Path,
) -> Result<(String, String), String> {
    // Validate the link name
    validate_workspace_link_name(link_name_input)?;
    let link_name = link_name_input.to_string();

    // Resolve the directory
    let resolved = resolve_existing_directory(link_path, cwd)?;
    let resolved_str = resolved
        .to_str()
        .ok_or_else(|| "Path contains invalid UTF-8".to_string())?
        .to_string();

    // Read the current view state
    let mut view_state = super::state_io::read_workspace_view_state(&selected.root)?;

    // Check that the link exists
    if !view_state.links.contains_key(&link_name) {
        return Err(format!("Unknown workspace link '{}'.", link_name));
    }

    // Update the link
    view_state.links.insert(link_name.clone(), Some(resolved_str.clone()));

    // Write the updated state
    super::state_io::write_workspace_view_state(&selected.root, &view_state)?;

    // Sync the open surface
    super::open_surface::sync_workspace_open_surface(&selected.root, &view_state, None)
        .map_err(|e| format!("Failed to sync workspace open surface: {}", e))?;

    Ok((link_name, resolved_str))
}

/// Status entry for a workspace or link, used in output reporting.
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceStatus {
    pub level: String,
    pub code: String,
    pub message: String,
}

impl WorkspaceStatus {
    pub fn warning(code: &str, message: &str) -> Self {
        WorkspaceStatus {
            level: "warning".to_string(),
            code: code.to_string(),
            message: message.to_string(),
        }
    }

    pub fn error(code: &str, message: &str) -> Self {
        WorkspaceStatus {
            level: "error".to_string(),
            code: code.to_string(),
            message: message.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_existing_directory_absolute() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let path_str = tmpdir.path().to_str().unwrap();

        let result = resolve_existing_directory(path_str, Path::new("/"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tmpdir.path().canonicalize().unwrap());
    }

    #[test]
    fn test_resolve_existing_directory_relative() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let cwd = tmpdir.path();
        let subdir = cwd.join("subdir");
        std::fs::create_dir(&subdir).expect("failed to create subdir");

        let result = resolve_existing_directory("subdir", cwd);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), subdir.canonicalize().unwrap());
    }

    #[test]
    fn test_resolve_existing_directory_empty() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let result = resolve_existing_directory("", tmpdir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must not be empty"));
    }

    #[test]
    fn test_resolve_existing_directory_nonexistent() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let result = resolve_existing_directory("/nonexistent/path/that/does/not/exist", tmpdir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not an existing folder"));
    }

    #[test]
    fn test_infer_link_name() {
        let path = Path::new("/home/user/my-repo");
        assert_eq!(infer_link_name(path), "my-repo");
    }

    #[test]
    fn test_add_workspace_link() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let ws_root = tmpdir.path();

        // Create the workspace structure
        let metadata_dir = super::super::foundation::get_workspace_metadata_dir(ws_root);
        std::fs::create_dir_all(&metadata_dir).expect("failed to create metadata dir");

        let view_yaml = r#"version: 1
name: test-ws
context: null
links: {}
"#;
        let view_path = super::super::foundation::get_workspace_view_state_path(ws_root);
        std::fs::write(&view_path, view_yaml).expect("failed to write view.yaml");

        // Create a directory to link
        let link_tmpdir = TempDir::new().expect("failed to create link tempdir");
        let link_path = link_tmpdir.path().to_str().unwrap();

        let selected = SelectedWorkspace {
            name: "test-ws".to_string(),
            root: ws_root.to_path_buf(),
        };

        let result = add_workspace_link(&selected, link_path, None, ws_root);
        assert!(result.is_ok());
        let (link_name, _resolved_path) = result.unwrap();
        // The name should be inferred from the directory basename, which is a random TempDir name
        assert!(!link_name.is_empty());

        // Verify the link was written
        let updated_state = super::super::state_io::read_workspace_view_state(ws_root)
            .expect("failed to read view state");
        assert!(updated_state.links.contains_key(&link_name));
    }

    #[test]
    fn test_update_workspace_link_unknown() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let ws_root = tmpdir.path();

        // Create the workspace structure
        let metadata_dir = super::super::foundation::get_workspace_metadata_dir(ws_root);
        std::fs::create_dir_all(&metadata_dir).expect("failed to create metadata dir");

        let view_yaml = r#"version: 1
name: test-ws
context: null
links: {}
"#;
        let view_path = super::super::foundation::get_workspace_view_state_path(ws_root);
        std::fs::write(&view_path, view_yaml).expect("failed to write view.yaml");

        let link_tmpdir = TempDir::new().expect("failed to create link tempdir");
        let link_path = link_tmpdir.path().to_str().unwrap();

        let selected = SelectedWorkspace {
            name: "test-ws".to_string(),
            root: ws_root.to_path_buf(),
        };

        let result = update_workspace_link(&selected, "nonexistent", link_path, ws_root);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown workspace link"));
    }
}
