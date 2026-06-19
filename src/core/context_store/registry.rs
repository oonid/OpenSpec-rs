use std::path::Path;

use super::foundation::{
    get_context_store_registry_path, parse_registry_state, serialize_registry_state,
    write_file_atomically, BackendConfig, ContextStoreRegistryEntry, RegistryState,
};

/// Load the registry from disk. If the file does not exist, return an empty default registry.
///
/// `global_data_dir` is forwarded to the path helpers (see `foundation`): `None` uses the real
/// global data dir, `Some(dir)` an isolated one.
pub fn load_registry(global_data_dir: Option<&Path>) -> RegistryState {
    let registry_path = get_context_store_registry_path(global_data_dir);

    if !registry_path.exists() {
        return RegistryState {
            version: 1,
            stores: Default::default(),
        };
    }

    match std::fs::read_to_string(&registry_path) {
        Ok(content) => parse_registry_state(&content).unwrap_or_else(|_| RegistryState {
            version: 1,
            stores: Default::default(),
        }),
        Err(_) => RegistryState {
            version: 1,
            stores: Default::default(),
        },
    }
}

/// Save the registry to disk atomically.
pub fn save_registry(state: &RegistryState, global_data_dir: Option<&Path>) -> Result<(), String> {
    let registry_path = get_context_store_registry_path(global_data_dir);
    let content = serialize_registry_state(state)?;
    write_file_atomically(&registry_path, &content)
        .map_err(|e| format!("Failed to write registry: {}", e))
}

/// List all registry entries, sorted by id (BTreeMap already sorts).
pub fn list_registry_entries(state: &RegistryState) -> Vec<ContextStoreRegistryEntry> {
    state
        .stores
        .iter()
        .map(|(id, entry)| ContextStoreRegistryEntry {
            id: id.clone(),
            backend: entry.backend.clone(),
        })
        .collect()
}

/// Get the store root (local_path) for a backend.
pub fn get_store_root_for_backend(backend: &BackendConfig) -> String {
    match backend {
        BackendConfig::Git { local_path, .. } => local_path.clone(),
    }
}

/// Normalize a path for comparison (attempt to canonicalize if path exists, otherwise use as-is).
fn normalize_path_for_comparison(path: &str) -> String {
    match std::fs::canonicalize(path) {
        Ok(canonical) => canonical.to_string_lossy().to_string(),
        Err(_) => path.to_string(),
    }
}

/// Check for conflicts when registering a store.
/// Errors if:
/// - The id is already registered (even with same path)
/// - Another store already points at the same local_path
///
/// Same id + same path is allowed (idempotent re-registration).
pub fn assert_no_registered_store_conflict(
    state: &RegistryState,
    id: &str,
    local_path: &str,
) -> Result<(), String> {
    let next_path = normalize_path_for_comparison(local_path);

    for (store_id, entry) in &state.stores {
        let backend = &entry.backend;
        let entry_path = get_store_root_for_backend(backend);
        let normalized_entry_path = normalize_path_for_comparison(&entry_path);

        // If same id and same path, allow (idempotent)
        if store_id == id && normalized_entry_path == next_path {
            continue;
        }

        // If same id but different path, error
        if store_id == id {
            return Err(format!(
                "Context store '{}' is already registered at {}.",
                id, entry_path
            ));
        }

        // If different id but same path, error
        if normalized_entry_path == next_path {
            return Err(format!(
                "Context store path is already registered as '{}'.",
                store_id
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::context_store::foundation::RegistryEntryState;
    use std::collections::BTreeMap;

    fn create_git_backend(local_path: &str, remote: Option<&str>, branch: Option<&str>) -> BackendConfig {
        BackendConfig::Git {
            local_path: local_path.to_string(),
            remote: remote.map(|s| s.to_string()),
            branch: branch.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_load_registry_nonexistent() {
        // An isolated, empty global data dir has no registry file → load returns empty default.
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let state = load_registry(Some(temp_dir.path()));
        assert_eq!(state.version, 1);
        assert!(state.stores.is_empty());
    }

    #[test]
    fn test_list_registry_entries() {
        let mut stores = BTreeMap::new();
        stores.insert(
            "z-store".to_string(),
            RegistryEntryState {
                backend: create_git_backend("/z/path", None, None),
            },
        );
        stores.insert(
            "a-store".to_string(),
            RegistryEntryState {
                backend: create_git_backend("/a/path", None, None),
            },
        );

        let state = RegistryState {
            version: 1,
            stores,
        };

        let entries = list_registry_entries(&state);
        assert_eq!(entries.len(), 2);
        // BTreeMap iteration is sorted, so a-store should come first
        assert_eq!(entries[0].id, "a-store");
        assert_eq!(entries[1].id, "z-store");
    }

    #[test]
    fn test_get_store_root_for_backend() {
        let backend = create_git_backend("/path/to/store", Some("https://example.com"), Some("main"));
        let root = get_store_root_for_backend(&backend);
        assert_eq!(root, "/path/to/store");
    }

    #[test]
    fn test_assert_no_conflict_idempotent_same_id_same_path() {
        let mut stores = BTreeMap::new();
        stores.insert(
            "my-store".to_string(),
            RegistryEntryState {
                backend: create_git_backend("/existing/path", None, None),
            },
        );
        let state = RegistryState {
            version: 1,
            stores,
        };

        // Re-registering same id with same path should be OK (idempotent)
        let result = assert_no_registered_store_conflict(&state, "my-store", "/existing/path");
        assert!(result.is_ok());
    }

    #[test]
    fn test_assert_no_conflict_same_id_different_path() {
        let mut stores = BTreeMap::new();
        stores.insert(
            "my-store".to_string(),
            RegistryEntryState {
                backend: create_git_backend("/existing/path", None, None),
            },
        );
        let state = RegistryState {
            version: 1,
            stores,
        };

        // Same id with different path should error
        let result = assert_no_registered_store_conflict(&state, "my-store", "/different/path");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already registered"));
    }

    #[test]
    fn test_assert_no_conflict_different_id_same_path() {
        let mut stores = BTreeMap::new();
        stores.insert(
            "store-a".to_string(),
            RegistryEntryState {
                backend: create_git_backend("/shared/path", None, None),
            },
        );
        let state = RegistryState {
            version: 1,
            stores,
        };

        // Different id with same path should error
        let result = assert_no_registered_store_conflict(&state, "store-b", "/shared/path");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already registered as"));
    }

    #[test]
    fn test_assert_no_conflict_distinct() {
        let mut stores = BTreeMap::new();
        stores.insert(
            "store-a".to_string(),
            RegistryEntryState {
                backend: create_git_backend("/path/a", None, None),
            },
        );
        let state = RegistryState {
            version: 1,
            stores,
        };

        // Different id with different path should be OK
        let result = assert_no_registered_store_conflict(&state, "store-b", "/path/b");
        assert!(result.is_ok());
    }

    #[test]
    fn test_save_and_load_registry() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let data_dir = Some(temp_dir.path());

        let mut stores = BTreeMap::new();
        stores.insert(
            "test-store".to_string(),
            RegistryEntryState {
                backend: create_git_backend("/tmp/test", Some("origin"), Some("main")),
            },
        );
        let original = RegistryState {
            version: 1,
            stores,
        };

        // Real round-trip through disk: save creates the registry under the isolated data dir,
        // load reads it back identically.
        save_registry(&original, data_dir).expect("save failed");
        assert!(get_context_store_registry_path(data_dir).exists());

        let loaded = load_registry(data_dir);
        assert_eq!(loaded, original);
    }
}
