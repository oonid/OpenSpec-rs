use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::foundation::{validate_workspace_link_name, PreferredOpener, WorkspaceViewState};
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

/// Result of creating a managed workspace
#[derive(Debug, Clone)]
pub struct WorkspaceSetupResult {
    pub name: String,
    pub root: PathBuf,
}

/// Parse workspace link inputs, supporting both `<path>` and `<name>=<path>` forms.
/// Returns a BTreeMap of link_name -> link_path. Validates for duplicates and empty paths.
pub fn parse_setup_links(link_inputs: &[String], cwd: &Path) -> Result<BTreeMap<String, String>, String> {
    let mut links = BTreeMap::new();

    for input in link_inputs {
        let (name, path_input) = if let Some(eq_pos) = input.find('=') {
            let name = input[..eq_pos].to_string();
            let path = input[eq_pos + 1..].to_string();
            (name, path)
        } else {
            // Infer name from the path
            let path = input.to_string();
            (String::new(), path)
        };

        // Validate the link name
        let link_name = if name.is_empty() {
            // Infer from path
            let resolved = resolve_existing_directory(&path_input, cwd)?;
            infer_link_name(&resolved)
        } else {
            validate_workspace_link_name(&name)?;
            name
        };

        // Check for duplicates
        if links.contains_key(&link_name) {
            return Err(format!("Duplicate link name '{}'.", link_name));
        }

        // Resolve the directory
        let resolved = resolve_existing_directory(&path_input, cwd)?;
        let resolved_str = resolved
            .to_str()
            .ok_or_else(|| "Path contains invalid UTF-8".to_string())?
            .to_string();

        links.insert(link_name, resolved_str);
    }

    Ok(links)
}

/// Create a managed workspace with the given name and links.
/// Creates the workspace directory structure, writes the view state, syncs the open surface,
/// and registers it in the registry. Returns the workspace name and root path.
pub fn create_managed_workspace(
    name: &str,
    links: BTreeMap<String, String>,
    preferred_opener: Option<PreferredOpener>,
    tools: Option<Vec<String>>,
    gdd: Option<&Path>,
) -> Result<WorkspaceSetupResult, String> {
    // Validate the workspace name
    super::foundation::validate_workspace_name(name)?;

    // Get the managed workspaces directory
    let managed_ws_dir = super::registry::get_managed_workspaces_dir(gdd);
    let workspace_root = managed_ws_dir.join(name);

    // Check if workspace already exists
    if workspace_root.exists() {
        let root_str = workspace_root
            .to_str()
            .unwrap_or("<invalid path>");
        return Err(format!(
            "Workspace '{}' already exists at {}.",
            name, root_str
        ));
    }

    // Create the workspace root directory
    std::fs::create_dir_all(&workspace_root)
        .map_err(|e| format!("Failed to create workspace directory: {}", e))?;

    // Build the WorkspaceViewState
    let view_state = WorkspaceViewState {
        version: 1,
        name: name.to_string(),
        context: None,
        links: {
            let mut m = BTreeMap::new();
            for (link_name, link_path) in links {
                m.insert(link_name, Some(link_path));
            }
            m
        },
        preferred_opener,
        tools,
        workspace_skills: None,
    };

    // Write the view state
    super::state_io::write_workspace_view_state(&workspace_root, &view_state)
        .map_err(|e| format!("Failed to write workspace state: {}", e))?;

    // Sync the open surface
    super::open_surface::sync_workspace_open_surface(&workspace_root, &view_state, None)
        .map_err(|e| format!("Failed to sync workspace open surface: {}", e))?;

    // Load the registry and add this workspace
    let mut registry = super::registry::load_workspace_registry(gdd)
        .map_err(|e| format!("Failed to load workspace registry: {}", e))?;

    let workspace_root_str = workspace_root
        .to_str()
        .ok_or_else(|| "Workspace path contains invalid UTF-8".to_string())?
        .to_string();

    registry
        .workspaces
        .insert(name.to_string(), workspace_root_str);

    // Save the updated registry
    super::registry::save_workspace_registry(&registry, gdd)
        .map_err(|e| format!("Failed to save workspace registry: {}", e))?;

    Ok(WorkspaceSetupResult {
        name: name.to_string(),
        root: workspace_root,
    })
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

    #[test]
    fn test_parse_setup_links_single_path() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let link_dir = tmpdir.path().join("myrepo");
        std::fs::create_dir(&link_dir).expect("failed to create link dir");
        let cwd = tmpdir.path();

        let inputs = vec!["myrepo".to_string()];
        let result = parse_setup_links(&inputs, cwd);

        assert!(result.is_ok());
        let links = result.unwrap();
        assert_eq!(links.len(), 1);
        assert!(links.contains_key("myrepo"));
    }

    #[test]
    fn test_parse_setup_links_named_path() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let link_dir = tmpdir.path().join("somepath");
        std::fs::create_dir(&link_dir).expect("failed to create link dir");
        let cwd = tmpdir.path();

        let inputs = vec!["mylink=somepath".to_string()];
        let result = parse_setup_links(&inputs, cwd);

        assert!(result.is_ok());
        let links = result.unwrap();
        assert_eq!(links.len(), 1);
        assert!(links.contains_key("mylink"));
    }

    #[test]
    fn test_parse_setup_links_duplicate_error() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let link_dir1 = tmpdir.path().join("path1");
        let link_dir2 = tmpdir.path().join("path2");
        std::fs::create_dir(&link_dir1).expect("failed to create link dir 1");
        std::fs::create_dir(&link_dir2).expect("failed to create link dir 2");
        let cwd = tmpdir.path();

        let inputs = vec!["mylink=path1".to_string(), "mylink=path2".to_string()];
        let result = parse_setup_links(&inputs, cwd);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate link name"));
    }

    #[test]
    fn test_parse_setup_links_nonexistent_error() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let cwd = tmpdir.path();

        let inputs = vec!["nonexistent".to_string()];
        let result = parse_setup_links(&inputs, cwd);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not an existing folder"));
    }

    #[test]
    fn test_create_managed_workspace() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let gdd = tmpdir.path();

        // Create a directory to link
        let link_dir = tmpdir.path().join("myrepo");
        std::fs::create_dir(&link_dir).expect("failed to create link dir");

        let mut links = BTreeMap::new();
        links.insert(
            "repo".to_string(),
            link_dir.to_str().unwrap().to_string(),
        );

        let result = create_managed_workspace("test-ws", links, None, None, Some(gdd));

        assert!(result.is_ok());
        let setup_result = result.unwrap();
        assert_eq!(setup_result.name, "test-ws");
        assert!(setup_result.root.exists());

        // Verify workspace structure
        let view_path = super::super::foundation::get_workspace_view_state_path(&setup_result.root);
        assert!(view_path.exists());

        // Verify registry entry
        let registry = super::super::registry::load_workspace_registry(Some(gdd))
            .expect("failed to load registry");
        assert!(registry.workspaces.contains_key("test-ws"));
    }

    #[test]
    fn test_create_managed_workspace_invalid_name() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let gdd = tmpdir.path();

        let links = BTreeMap::new();

        // Try with invalid workspace name (contains uppercase)
        let result = create_managed_workspace("Invalid-Name", links.clone(), None, None, Some(gdd));
        assert!(result.is_err());

        // Try with empty name
        let result = create_managed_workspace("", links, None, None, Some(gdd));
        assert!(result.is_err());
    }

    #[test]
    fn test_create_managed_workspace_with_tools() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let gdd = tmpdir.path();

        let links = BTreeMap::new();
        let tools = Some(vec!["claude".to_string(), "cursor".to_string()]);

        let result = create_managed_workspace("ws-with-tools", links, None, tools, Some(gdd));

        assert!(result.is_ok());
        let setup_result = result.unwrap();

        // Verify tools were written
        let view_state = super::super::state_io::read_workspace_view_state(&setup_result.root)
            .expect("failed to read view state");
        assert!(view_state.tools.is_some());
        assert_eq!(view_state.tools.unwrap(), vec!["claude", "cursor"]);
    }

    #[test]
    fn test_parse_setup_links_multiple_mixed() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let repo1 = tmpdir.path().join("repo1");
        let repo2 = tmpdir.path().join("repo2");
        std::fs::create_dir(&repo1).expect("failed to create repo1");
        std::fs::create_dir(&repo2).expect("failed to create repo2");
        let cwd = tmpdir.path();

        // Mix of simple paths and named paths
        let inputs = vec!["repo1".to_string(), "alias=repo2".to_string()];
        let result = parse_setup_links(&inputs, cwd);

        assert!(result.is_ok());
        let links = result.unwrap();
        assert_eq!(links.len(), 2);
        assert!(links.contains_key("repo1"));
        assert!(links.contains_key("alias"));
    }
}
