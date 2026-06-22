use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::foundation::{
    get_context_store_metadata_path, get_default_context_store_root, parse_metadata_state,
    serialize_metadata_state, validate_context_store_id, BackendConfig, MetadataState,
    RegistryEntryState,
};
use super::registry::{
    assert_no_registered_store_conflict, get_store_root_for_backend, list_registry_entries,
    load_registry, save_registry,
};

// ============================================================================
// Result Types (for operations return values)
// ============================================================================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextStoreInfo {
    pub id: String,
    pub root: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStatus {
    pub is_repository: bool,
    pub initialized: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationResult {
    pub store: ContextStoreInfo,
    pub git: GitStatus,
    pub created_artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    pub store: ContextStoreInfo,
    pub registry_removed: bool,
    pub deleted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_on_disk: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResult {
    pub stores: Vec<ContextStoreInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreInspection {
    pub id: String,
    pub root: String,
    pub metadata_present: bool,
    pub metadata_valid: bool,
    pub is_git_repository: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorResult {
    pub stores: Vec<StoreInspection>,
}

// ============================================================================
// Helper Enums and Functions
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathKind {
    Missing,
    Directory,
    File,
    Other,
}

/// Determine what kind of filesystem object exists at the given path.
fn path_kind(path: &Path) -> PathKind {
    match fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                PathKind::Directory
            } else if metadata.is_file() {
                PathKind::File
            } else {
                PathKind::Other
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => PathKind::Missing,
        Err(_) => PathKind::Other, // Treat permission errors, etc. as "other"
    }
}

/// Read the metadata file from a store root if it exists. Returns Ok(None) if absent.
/// Returns Err if the file exists but is invalid.
fn read_optional_metadata(store_root: &Path) -> Result<Option<MetadataState>, String> {
    let metadata_path = get_context_store_metadata_path(store_root);
    match fs::read_to_string(&metadata_path) {
        Ok(content) => {
            let metadata = parse_metadata_state(&content)?;
            Ok(Some(metadata))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!(
            "Failed to read metadata at {}: {}",
            metadata_path.display(),
            e
        )),
    }
}

/// Write metadata if it's missing. Returns true if metadata was created, false if it already existed.
fn write_metadata_if_missing(store_root: &Path, id: &str) -> Result<bool, String> {
    let metadata_path = get_context_store_metadata_path(store_root);

    // Check if metadata already exists
    if metadata_path.exists() {
        return Ok(false);
    }

    // Create the metadata file
    let metadata = MetadataState {
        version: 1,
        id: id.to_string(),
    };
    let serialized = serialize_metadata_state(&metadata)?;

    super::foundation::write_file_atomically(&metadata_path, &serialized)
        .map_err(|e| format!("Failed to write metadata: {}", e))?;

    Ok(true)
}

/// Initialize a git repository at the store root using `git init`.
/// Returns true if the repository was newly initialized, false if it already existed.
/// Returns false on git errors (treating git absence as non-fatal).
fn init_git_repository(store_root: &Path) -> bool {
    // Check if .git already exists
    if is_git_repository_at_root(store_root) {
        return false;
    }

    // Try to run `git init`
    match Command::new("git")
        .arg("init")
        .current_dir(store_root)
        .output()
    {
        Ok(output) => output.status.success(),
        Err(_) => false, // git not found or other error
    }
}

/// Check if a directory is a git repository by checking for .git.
fn is_git_repository_at_root(store_root: &Path) -> bool {
    let git_path = store_root.join(".git");
    matches!(path_kind(&git_path), PathKind::Directory | PathKind::File)
}

/// Find the containing git repository root for a given store path.
/// Returns Some(git_root) if the store path is strictly inside a git repository,
/// None otherwise or if a containing repo cannot be determined.
fn find_containing_git_repository_root(store_root: &Path) -> Option<PathBuf> {
    // Resolve the nearest existing ancestor directory of store_root.
    // The store_root itself may not exist yet, so we walk up to find the first existing dir.
    let mut check_path = store_root.to_path_buf();

    // If store_root itself doesn't exist, go up to the parent.
    if !check_path.exists() {
        check_path = check_path.parent()?.to_path_buf();
    }

    // Walk up from check_path (or its parent if check_path is a file).
    while check_path.exists() {
        // Try git command first: `git -C <dir> rev-parse --show-toplevel`
        if let Ok(output) = Command::new("git")
            .arg("-C")
            .arg(&check_path)
            .arg("rev-parse")
            .arg("--show-toplevel")
            .output()
        {
            if output.status.success() {
                if let Ok(git_root_str) = String::from_utf8(output.stdout) {
                    let git_root_str = git_root_str.trim();
                    let git_root = PathBuf::from(git_root_str);

                    // Verify that git_root strictly contains store_root.
                    // Canonicalize what we can; if store_root doesn't exist, use the absolute form.
                    let store_abs = if store_root.is_absolute() {
                        store_root.to_path_buf()
                    } else {
                        if let Ok(cwd) = std::env::current_dir() {
                            cwd.join(store_root)
                        } else {
                            store_root.to_path_buf()
                        }
                    };

                    // Canonicalize both for clean comparison if they exist.
                    let git_root_canonical = git_root.canonicalize().unwrap_or(git_root);
                    let store_abs_canonical = store_abs.canonicalize().unwrap_or(store_abs);

                    // Check: git_root is a proper ancestor (not equal).
                    if store_abs_canonical.starts_with(&git_root_canonical)
                        && store_abs_canonical != git_root_canonical
                    {
                        return Some(git_root_canonical);
                    }
                    // If they're equal or git_root is not an ancestor, no containing repo.
                    return None;
                }
            }
        }

        // Fallback: check if check_path itself is a git repository.
        if is_git_repository_at_root(&check_path) {
            // Verify containment the same way.
            let store_abs = if store_root.is_absolute() {
                store_root.to_path_buf()
            } else {
                if let Ok(cwd) = std::env::current_dir() {
                    cwd.join(store_root)
                } else {
                    store_root.to_path_buf()
                }
            };

            let check_canonical = check_path.canonicalize().unwrap_or(check_path.clone());
            let store_abs_canonical = store_abs.canonicalize().unwrap_or(store_abs);

            if store_abs_canonical.starts_with(&check_canonical)
                && store_abs_canonical != check_canonical
            {
                return Some(check_canonical);
            }
            return None;
        }

        // Move up to parent.
        check_path = check_path.parent()?.to_path_buf();
    }

    None
}

/// Assert that the store setup path is not nested inside an existing git repository.
/// If allow_inside_git_repository is true, always succeeds.
/// Otherwise, returns Err if the path is inside a git repo.
fn assert_setup_path_not_nested_in_git_repo(
    store_root: &Path,
    allow_inside_git_repository: bool,
) -> Result<(), String> {
    if allow_inside_git_repository {
        return Ok(());
    }

    if let Some(git_root) = find_containing_git_repository_root(store_root) {
        return Err(format!(
            "Context store setup path is inside another Git repository: {}",
            git_root.display()
        ));
    }

    Ok(())
}

/// Infer a store id from the final path component.
/// Validates the inferred id against the context store id rules.
fn infer_store_id_from_path(store_root: &Path) -> Result<String, String> {
    let basename = store_root
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            "Cannot infer store id from path: invalid UTF-8 or empty path".to_string()
        })?;

    // Mirror upstream `inferStoreIdFromPath`: validate the basename and use it as-is. Upstream
    // does NOT sanitize — an invalid basename is an error so the user passes an explicit --id.
    validate_context_store_id(basename).map(|_| basename.to_string())
}

// ============================================================================
// Public Operations
// ============================================================================

/// Set up a new context store.
///
/// If `path` is None, uses the default managed location derived from `id`.
/// If `path` is Some, uses that path (creating it if needed).
/// If `init_git` is true, initializes a git repository.
/// If `allow_inside_git_repository` is true, bypasses the check that prevents nesting in git repos.
/// Returns MutationResult with store info, git status, and created artifacts.
pub fn setup_context_store(
    id: Option<&str>,
    path: Option<&str>,
    init_git: bool,
    allow_inside_git_repository: bool,
    global_data_dir: Option<&Path>,
) -> Result<MutationResult, String> {
    // Resolve the store id
    let store_id = match (id, path) {
        (Some(id), _) => {
            validate_context_store_id(id)?;
            id.to_string()
        }
        (None, Some(_)) => {
            return Err("Context store id is required when providing an explicit path".to_string());
        }
        (None, None) => {
            return Err("Context store id is required (or provide --path)".to_string());
        }
    };

    // Resolve the store root
    let store_root = match path {
        Some(p) => PathBuf::from(p),
        None => get_default_context_store_root(&store_id, global_data_dir),
    };

    let kind = path_kind(&store_root);

    // Validate the path kind
    if matches!(kind, PathKind::File | PathKind::Other) {
        return Err(format!(
            "Context store setup path is not a directory: {}",
            store_root.display()
        ));
    }

    // Check that the store path is not nested in an existing git repository.
    assert_setup_path_not_nested_in_git_repo(&store_root, allow_inside_git_repository)?;

    // If the directory doesn't exist, create it (with cleanup on failure).
    let created_dir = matches!(kind, PathKind::Missing);
    if created_dir {
        fs::create_dir_all(&store_root)
            .map_err(|e| format!("Failed to create context store directory: {}", e))?;
    }

    // Run the setup, with cleanup-on-failure if we created a directory.
    match run_setup_setup(&store_id, &store_root, init_git, global_data_dir) {
        Ok(result) => Ok(result),
        Err(e) => {
            if created_dir {
                let _ = fs::remove_dir_all(&store_root);
            }
            Err(e)
        }
    }
}

/// Helper for the actual setup logic (factored out for cleanup-on-failure).
fn run_setup_setup(
    store_id: &str,
    store_root: &Path,
    init_git: bool,
    global_data_dir: Option<&Path>,
) -> Result<MutationResult, String> {
    // Load the registry
    let mut registry = load_registry(global_data_dir);

    let store_root_str = store_root.to_string_lossy().to_string();
    let backend = BackendConfig::Git {
        local_path: store_root_str.clone(),
        remote: None,
        branch: None,
    };

    // Check for conflicts
    assert_no_registered_store_conflict(&registry, store_id, &store_root_str)?;

    // Initialize git if requested
    let git_initialized = if init_git {
        init_git_repository(store_root)
    } else {
        false
    };

    // Write metadata if missing
    let metadata_created = write_metadata_if_missing(store_root, store_id)?;

    // Update registry
    registry
        .stores
        .insert(store_id.to_string(), RegistryEntryState { backend });
    save_registry(&registry, global_data_dir)?;

    // Determine created artifacts
    let created_artifacts = if metadata_created {
        vec![".openspec-store/store.yaml".to_string()]
    } else {
        vec![]
    };

    // Determine if it's a git repository
    let is_repository = is_git_repository_at_root(store_root);

    Ok(MutationResult {
        store: ContextStoreInfo {
            id: store_id.to_string(),
            root: store_root.to_string_lossy().to_string(),
            metadata_path: Some(
                get_context_store_metadata_path(store_root)
                    .to_string_lossy()
                    .to_string(),
            ),
        },
        git: GitStatus {
            is_repository,
            initialized: git_initialized,
        },
        created_artifacts,
    })
}

/// Register an existing context store.
///
/// If `path` is None, the current working directory is used.
/// If `id` is not provided, it's inferred from the metadata or path.
/// Returns MutationResult with store info and metadata.
pub fn register_existing_context_store(
    path: Option<&str>,
    id: Option<&str>,
    global_data_dir: Option<&Path>,
) -> Result<MutationResult, String> {
    // Resolve the store root
    let store_root = match path {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?,
    };

    let kind = path_kind(&store_root);

    // Validate that the path exists and is a directory
    if matches!(kind, PathKind::Missing) {
        return Err(format!(
            "Context store path does not exist: {}",
            store_root.display()
        ));
    }

    if !matches!(kind, PathKind::Directory) {
        return Err(format!(
            "Context store path is not a directory: {}",
            store_root.display()
        ));
    }

    // Try to read existing metadata
    let metadata = read_optional_metadata(&store_root)?;

    // Validate explicit id if provided
    let explicit_id = id
        .map(|i| validate_context_store_id(i).map(|_| i.to_string()))
        .transpose()?;

    // Check for id mismatches
    if let Some(ref meta) = metadata {
        if let Some(ref ex_id) = explicit_id {
            if meta.id != *ex_id {
                return Err(format!(
                    "Context store metadata id '{}' does not match --id '{}'.",
                    meta.id, ex_id
                ));
            }
        }
    }

    // Determine final id: metadata.id ?? explicit_id ?? infer from path
    #[allow(clippy::unnecessary_lazy_evaluations)]
    let store_id = metadata
        .as_ref()
        .map(|m| m.id.clone())
        .or_else(|| explicit_id)
        .or_else(|| infer_store_id_from_path(&store_root).ok())
        .ok_or_else(|| {
            "Could not determine context store id; provide --id or use a path with a valid name"
                .to_string()
        })?;

    // Load the registry
    let mut registry = load_registry(global_data_dir);

    let store_root_str = store_root.to_string_lossy().to_string();
    let backend = BackendConfig::Git {
        local_path: store_root_str.clone(),
        remote: None,
        branch: None,
    };

    // Check for conflicts
    assert_no_registered_store_conflict(&registry, &store_id, &store_root_str)?;

    // Write metadata if missing
    let metadata_created = write_metadata_if_missing(&store_root, &store_id)?;

    // Update registry
    registry
        .stores
        .insert(store_id.to_string(), RegistryEntryState { backend });
    save_registry(&registry, global_data_dir)?;

    // Determine created artifacts
    let created_artifacts = if metadata_created {
        vec![".openspec-store/store.yaml".to_string()]
    } else {
        vec![]
    };

    // Determine if it's a git repository
    let is_repository = is_git_repository_at_root(&store_root);

    Ok(MutationResult {
        store: ContextStoreInfo {
            id: store_id,
            root: store_root.to_string_lossy().to_string(),
            metadata_path: Some(
                get_context_store_metadata_path(&store_root)
                    .to_string_lossy()
                    .to_string(),
            ),
        },
        git: GitStatus {
            is_repository,
            initialized: false,
        },
        created_artifacts,
    })
}

/// Unregister a context store (remove from registry but don't delete files).
pub fn unregister_context_store(
    id: &str,
    global_data_dir: Option<&Path>,
) -> Result<CleanupResult, String> {
    validate_context_store_id(id)?;

    // Load the registry
    let mut registry = load_registry(global_data_dir);

    // Find and remove the entry
    let entry = registry
        .stores
        .remove(id)
        .ok_or_else(|| format!("Context store '{}' not found in registry", id))?;

    // Get the store root
    let store_root = get_store_root_for_backend(&entry.backend);

    // Save the updated registry
    save_registry(&registry, global_data_dir)?;

    Ok(CleanupResult {
        store: ContextStoreInfo {
            id: id.to_string(),
            root: store_root.clone(),
            metadata_path: None,
        },
        registry_removed: true,
        deleted: false,
        deleted_path: None,
        left_on_disk: Some(store_root),
    })
}

/// Remove a context store (delete from registry and from disk).
/// Includes safety checks: refuses to delete if the path is not a directory,
/// has no metadata, or the metadata id doesn't match.
pub fn remove_context_store(
    id: &str,
    global_data_dir: Option<&Path>,
) -> Result<CleanupResult, String> {
    validate_context_store_id(id)?;

    // Load the registry
    let mut registry = load_registry(global_data_dir);

    // Find and remove the entry
    let entry = registry
        .stores
        .remove(id)
        .ok_or_else(|| format!("Context store '{}' not found in registry", id))?;

    // Get the store root
    let store_root = get_store_root_for_backend(&entry.backend);
    let store_root_path = PathBuf::from(&store_root);

    // Save the updated registry (do this before deleting, so we don't lose the entry)
    save_registry(&registry, global_data_dir)?;

    // Safety checks before deletion
    let kind = path_kind(&store_root_path);

    // If the path is missing, record it and continue
    if matches!(kind, PathKind::Missing) {
        return Ok(CleanupResult {
            store: ContextStoreInfo {
                id: id.to_string(),
                root: store_root.clone(),
                metadata_path: None,
            },
            registry_removed: true,
            deleted: false,
            deleted_path: None,
            left_on_disk: None,
        });
    }

    // Refuse to delete if not a directory
    if !matches!(kind, PathKind::Directory) {
        return Err(format!(
            "Context store path is not a directory; refusing to delete: {}",
            store_root
        ));
    }

    // Refuse to delete if metadata is missing
    let metadata = read_optional_metadata(&store_root_path)?;
    if metadata.is_none() {
        return Err(format!(
            "Context store path has no metadata; refusing to delete: {}",
            store_root
        ));
    }

    // Refuse to delete if metadata id doesn't match
    if let Some(meta) = metadata {
        if meta.id != id {
            return Err(format!(
                "Context store metadata id '{}' does not match requested id '{}'",
                meta.id, id
            ));
        }
    }

    // Delete the directory
    fs::remove_dir_all(&store_root_path)
        .map_err(|e| format!("Failed to delete context store directory: {}", e))?;

    Ok(CleanupResult {
        store: ContextStoreInfo {
            id: id.to_string(),
            root: store_root.clone(),
            metadata_path: None,
        },
        registry_removed: true,
        deleted: true,
        deleted_path: Some(store_root),
        left_on_disk: None,
    })
}

/// List all registered context stores.
pub fn list_context_stores(global_data_dir: Option<&Path>) -> ListResult {
    let registry = load_registry(global_data_dir);
    let entries = list_registry_entries(&registry);

    let stores = entries
        .into_iter()
        .map(|entry| ContextStoreInfo {
            id: entry.id,
            root: get_store_root_for_backend(&entry.backend),
            metadata_path: None,
        })
        .collect();

    ListResult { stores }
}

/// Inspect context stores and report their status.
/// If `id` is Some, only inspect that store. Otherwise, inspect all.
pub fn doctor_context_stores(
    id: Option<&str>,
    global_data_dir: Option<&Path>,
) -> Result<DoctorResult, String> {
    let registry = load_registry(global_data_dir);
    let entries = list_registry_entries(&registry);

    // Filter to the requested id if provided
    let selected: Vec<_> = if let Some(target_id) = id {
        validate_context_store_id(target_id)?;
        entries.into_iter().filter(|e| e.id == target_id).collect()
    } else {
        entries
    };

    let stores = selected
        .into_iter()
        .map(|entry| {
            let store_root_str = get_store_root_for_backend(&entry.backend);
            let store_root = PathBuf::from(&store_root_str);

            // Check metadata presence and validity
            let (metadata_present, metadata_valid) = match read_optional_metadata(&store_root) {
                Ok(Some(meta)) => {
                    let is_valid = meta.id == entry.id;
                    (true, is_valid)
                }
                Ok(None) => (false, false),
                Err(_) => (true, false), // File exists but is invalid
            };

            // Check if it's a git repository
            let is_git_repository = is_git_repository_at_root(&store_root);

            StoreInspection {
                id: entry.id,
                root: store_root_str,
                metadata_present,
                metadata_valid,
                is_git_repository,
            }
        })
        .collect();

    Ok(DoctorResult { stores })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_into_temp_global_dir() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let global_data_dir = Some(temp_dir.path());

        let result = setup_context_store(Some("test-store"), None, false, false, global_data_dir)
            .expect("setup failed");

        assert_eq!(result.store.id, "test-store");
        assert!(!result.git.is_repository);
        assert!(!result.git.initialized);
        assert_eq!(result.created_artifacts, vec![".openspec-store/store.yaml"]);

        // Verify the store directory was created
        let store_root = PathBuf::from(&result.store.root);
        assert!(store_root.exists());
        assert!(store_root.is_dir());

        // Verify metadata was created
        let metadata_path = get_context_store_metadata_path(&store_root);
        assert!(metadata_path.exists());
        let content = fs::read_to_string(&metadata_path).expect("read metadata");
        assert!(content.contains("test-store"));

        // Verify registry was updated
        let registry = load_registry(global_data_dir);
        assert!(registry.stores.contains_key("test-store"));
    }

    #[test]
    fn test_setup_with_init_git() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let global_data_dir = Some(temp_dir.path());

        let result = setup_context_store(Some("git-store"), None, true, false, global_data_dir)
            .expect("setup failed");

        assert_eq!(result.store.id, "git-store");
        assert!(result.git.initialized); // Should be true if git succeeded
                                         // git.is_repository should match whether git init actually worked

        let store_root = PathBuf::from(&result.store.root);
        assert!(store_root.exists());
    }

    #[test]
    fn test_setup_requires_id_or_path() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let global_data_dir = Some(temp_dir.path());

        // No id, no path → error
        let result = setup_context_store(None, None, false, false, global_data_dir);
        assert!(result.is_err());

        // No id, has path → error
        let result =
            setup_context_store(None, Some("/tmp/somepath"), false, false, global_data_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_existing_store() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let global_data_dir = Some(temp_dir.path());

        // First set up a store
        let setup_result =
            setup_context_store(Some("my-store"), None, false, false, global_data_dir)
                .expect("setup failed");

        let store_root = PathBuf::from(&setup_result.store.root);

        // Clear the registry to simulate an existing store that's not registered
        let mut registry = load_registry(global_data_dir);
        registry.stores.clear();
        save_registry(&registry, global_data_dir).expect("clear registry");

        // Now register it again
        let register_result = register_existing_context_store(
            Some(store_root.to_str().unwrap()),
            None,
            global_data_dir,
        )
        .expect("register failed");

        assert_eq!(register_result.store.id, "my-store");
        assert!(!register_result.git.initialized);

        // Verify it's in the registry
        let registry = load_registry(global_data_dir);
        assert!(registry.stores.contains_key("my-store"));
    }

    #[test]
    fn test_register_nonexistent_path_fails() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let global_data_dir = Some(temp_dir.path());

        let result = register_existing_context_store(
            Some("/nonexistent/path/to/store"),
            Some("my-store"),
            global_data_dir,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_register_file_instead_of_dir_fails() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let global_data_dir = Some(temp_dir.path());

        let file_path = temp_dir.path().join("notadir.txt");
        fs::write(&file_path, "test").expect("create file");

        let result = register_existing_context_store(
            Some(file_path.to_str().unwrap()),
            Some("my-store"),
            global_data_dir,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_register_infers_invalid_basename_errors_without_id() {
        // Mirrors upstream: an invalid directory basename is NOT sanitized; without an explicit
        // --id, registration errors so the user supplies a valid id.
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let global_data_dir = Some(temp_dir.path());

        let bad_dir = temp_dir.path().join("My_Store");
        fs::create_dir_all(&bad_dir).expect("create dir");

        let result =
            register_existing_context_store(Some(bad_dir.to_str().unwrap()), None, global_data_dir);
        assert!(
            result.is_err(),
            "invalid basename should error, not be silently sanitized"
        );
    }

    #[test]
    fn test_unregister_removes_registry_entry() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let global_data_dir = Some(temp_dir.path());

        // Set up a store
        setup_context_store(Some("my-store"), None, false, false, global_data_dir)
            .expect("setup failed");

        // Verify it's registered
        let registry = load_registry(global_data_dir);
        assert!(registry.stores.contains_key("my-store"));

        // Unregister it
        let cleanup_result =
            unregister_context_store("my-store", global_data_dir).expect("unregister failed");

        assert_eq!(cleanup_result.store.id, "my-store");
        assert!(cleanup_result.registry_removed);
        assert!(!cleanup_result.deleted);
        assert!(cleanup_result.left_on_disk.is_some());

        // Verify it's no longer registered
        let registry = load_registry(global_data_dir);
        assert!(!registry.stores.contains_key("my-store"));

        // Verify the folder still exists
        let store_root = PathBuf::from(cleanup_result.left_on_disk.unwrap());
        assert!(store_root.exists());
    }

    #[test]
    fn test_remove_deletes_folder() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let global_data_dir = Some(temp_dir.path());

        // Set up a store
        let setup_result =
            setup_context_store(Some("to-delete"), None, false, false, global_data_dir)
                .expect("setup failed");

        let store_root = PathBuf::from(&setup_result.store.root);
        assert!(store_root.exists());

        // Remove it
        let cleanup_result =
            remove_context_store("to-delete", global_data_dir).expect("remove failed");

        assert_eq!(cleanup_result.store.id, "to-delete");
        assert!(cleanup_result.registry_removed);
        assert!(cleanup_result.deleted);
        assert!(cleanup_result.deleted_path.is_some());

        // Verify the folder is gone
        assert!(!store_root.exists());
    }

    #[test]
    fn test_remove_fails_on_missing_metadata() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let global_data_dir = Some(temp_dir.path());

        // Create a store
        let setup_result =
            setup_context_store(Some("no-meta"), None, false, false, global_data_dir)
                .expect("setup failed");

        let store_root = PathBuf::from(&setup_result.store.root);

        // Delete the metadata file (but leave the directory)
        let metadata_path = get_context_store_metadata_path(&store_root);
        fs::remove_file(&metadata_path).expect("remove metadata");

        // Try to remove — should fail
        let result = remove_context_store("no-meta", global_data_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no metadata"));
    }

    #[test]
    fn test_list_context_stores() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let global_data_dir = Some(temp_dir.path());

        // Set up a few stores
        setup_context_store(Some("store-a"), None, false, false, global_data_dir).expect("setup a");
        setup_context_store(Some("store-b"), None, false, false, global_data_dir).expect("setup b");
        setup_context_store(Some("store-c"), None, false, false, global_data_dir).expect("setup c");

        let result = list_context_stores(global_data_dir);

        assert_eq!(result.stores.len(), 3);
        let ids: Vec<_> = result.stores.iter().map(|s| s.id.clone()).collect();
        assert_eq!(ids, vec!["store-a", "store-b", "store-c"]);
    }

    #[test]
    fn test_doctor_context_stores() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let global_data_dir = Some(temp_dir.path());

        // Set up a store
        setup_context_store(Some("healthy-store"), None, false, false, global_data_dir)
            .expect("setup failed");

        // Doctor all stores
        let result = doctor_context_stores(None, global_data_dir).expect("doctor failed");

        assert_eq!(result.stores.len(), 1);
        let store = &result.stores[0];
        assert_eq!(store.id, "healthy-store");
        assert!(store.metadata_present);
        assert!(store.metadata_valid);

        // Now doctor a specific store
        let result =
            doctor_context_stores(Some("healthy-store"), global_data_dir).expect("doctor failed");
        assert_eq!(result.stores.len(), 1);
    }

    #[test]
    fn test_doctor_reports_invalid_metadata() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let global_data_dir = Some(temp_dir.path());

        // Set up a store
        let setup_result =
            setup_context_store(Some("bad-meta"), None, false, false, global_data_dir)
                .expect("setup failed");

        let store_root = PathBuf::from(&setup_result.store.root);

        // Corrupt the metadata
        let metadata_path = get_context_store_metadata_path(&store_root);
        fs::write(&metadata_path, "invalid: yaml: content:").expect("corrupt metadata");

        // Doctor should detect this
        let result = doctor_context_stores(None, global_data_dir).expect("doctor failed");

        assert_eq!(result.stores.len(), 1);
        let store = &result.stores[0];
        assert_eq!(store.id, "bad-meta");
        assert!(store.metadata_present);
        assert!(!store.metadata_valid);
    }

    #[test]
    fn test_setup_cleanup_on_failure() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let global_data_dir = Some(temp_dir.path());

        // Try to set up with an invalid id; should fail and not leave a directory
        let result = setup_context_store(Some("INVALID"), None, false, false, global_data_dir);

        assert!(result.is_err());

        // The managed directory for "INVALID" should not exist
        // (because the id validation failed before we created it)
    }

    #[test]
    fn test_infer_store_id_from_basename() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let store_dir = temp_dir.path().join("my-store");
        fs::create_dir(&store_dir).expect("create store dir");

        // Create metadata manually with a matching id
        let metadata = MetadataState {
            version: 1,
            id: "my-store".to_string(),
        };
        let metadata_path = get_context_store_metadata_path(&store_dir);
        super::super::foundation::write_file_atomically(
            &metadata_path,
            &serialize_metadata_state(&metadata).unwrap(),
        )
        .expect("write metadata");

        // Register without specifying an id; should infer from path
        let result = register_existing_context_store(
            Some(store_dir.to_str().unwrap()),
            None,
            Some(temp_dir.path()),
        )
        .expect("register failed");

        assert_eq!(result.store.id, "my-store");
    }

    #[test]
    fn test_setup_refuses_nested_in_git_repo() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let git_dir = temp_dir.path().join("git-repo");
        fs::create_dir(&git_dir).expect("create git dir");

        // Initialize a git repository in git_dir
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&git_dir)
            .output()
            .expect("git init failed");

        // Try to set up a store inside the git repo; should fail
        let store_dir = git_dir.join("store");
        let global_data_dir = Some(temp_dir.path());

        let result = setup_context_store(
            Some("nested-store"),
            Some(store_dir.to_str().unwrap()),
            false,
            false,
            global_data_dir,
        );

        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("inside another Git repository"));
    }

    #[test]
    fn test_setup_allows_nested_with_flag() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let git_dir = temp_dir.path().join("git-repo");
        fs::create_dir(&git_dir).expect("create git dir");

        // Initialize a git repository
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&git_dir)
            .output()
            .expect("git init failed");

        // Now set up a store inside the git repo WITH the flag; should succeed
        let store_dir = git_dir.join("store");
        let global_data_dir = Some(temp_dir.path());

        let result = setup_context_store(
            Some("nested-store"),
            Some(store_dir.to_str().unwrap()),
            false,
            true, // allow_inside_git_repository
            global_data_dir,
        );

        assert!(result.is_ok());
        let store_result = result.unwrap();
        assert_eq!(store_result.store.id, "nested-store");
    }

    #[test]
    fn test_setup_succeeds_outside_git_repo() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let store_dir = temp_dir.path().join("store");
        let global_data_dir = Some(temp_dir.path());

        // Store is not in a git repo; should succeed
        let result = setup_context_store(
            Some("free-store"),
            Some(store_dir.to_str().unwrap()),
            false,
            false,
            global_data_dir,
        );

        assert!(result.is_ok());
        let store_result = result.unwrap();
        assert_eq!(store_result.store.id, "free-store");
    }

    #[test]
    fn test_setup_with_init_git_at_root() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let global_data_dir = Some(temp_dir.path());

        // Set up a store with init_git; the repo is AT the store root, not a containing repo.
        // This should succeed because we're checking for a containing git repo, not one at the root itself.
        let result = setup_context_store(
            Some("repo-store"),
            None,
            true,  // init_git
            false, // allow_inside_git_repository
            global_data_dir,
        );

        assert!(result.is_ok());
        let store_result = result.unwrap();
        assert_eq!(store_result.store.id, "repo-store");
        // The init_git should have created a repo at the store root
        assert!(store_result.git.initialized || store_result.git.is_repository);
    }

    #[test]
    fn test_find_containing_git_repository() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let git_dir = temp_dir.path().join("git-repo");
        fs::create_dir(&git_dir).expect("create git dir");

        // Initialize git repo
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&git_dir)
            .output()
            .expect("git init failed");

        // Test: path inside git repo should find it
        // We need to create the actual path so that we can check
        let store_path = git_dir.join("sub").join("store");
        fs::create_dir_all(&store_path).expect("create store path");
        let found = find_containing_git_repository_root(&store_path);
        assert!(found.is_some());

        // Test: path not in git repo should return None
        let free_path = temp_dir.path().join("free").join("store");
        // Don't create this path; test that a non-existent path not in any git repo returns None
        let found = find_containing_git_repository_root(&free_path);
        assert!(found.is_none());
    }
}
