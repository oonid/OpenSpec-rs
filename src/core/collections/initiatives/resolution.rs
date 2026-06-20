use std::path::{Path, PathBuf};

use crate::core::collections::initiatives::operations::read_initiative;
use crate::core::context_store::{
    get_context_store_metadata_path, get_store_root_for_backend, list_registry_entries,
    load_registry, parse_metadata_state,
};

use super::schema::InitiativeState;

/// A selected context store with its id and root path.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectedStore {
    pub id: String,
    pub root: PathBuf,
}

/// Get a list of registered context stores.
pub fn registered_stores(gdd: Option<&Path>) -> Vec<SelectedStore> {
    let registry = load_registry(gdd);
    let entries = list_registry_entries(&registry);
    entries
        .into_iter()
        .map(|entry| SelectedStore {
            id: entry.id,
            root: PathBuf::from(get_store_root_for_backend(&entry.backend)),
        })
        .collect()
}

/// Resolve a selected context store given optional --store id and --store-path.
///
/// If both are provided, returns an error (selector conflict).
/// If --store is provided, looks it up in the registry.
/// If --store-path is provided, reads the metadata id (or uses basename as fallback).
/// If neither is provided, returns an error.
pub fn resolve_selected_store(
    store: Option<&str>,
    store_path: Option<&str>,
    gdd: Option<&Path>,
) -> Result<SelectedStore, String> {
    // Check for selector conflict
    if store.is_some() && store_path.is_some() {
        return Err("Pass only one of --store or --store-path.".to_string());
    }

    if let Some(id) = store {
        // Look up the store in the registry
        let registry = load_registry(gdd);
        let entries = list_registry_entries(&registry);
        for entry in entries {
            if entry.id == id {
                let root = PathBuf::from(get_store_root_for_backend(&entry.backend));
                return Ok(SelectedStore {
                    id: entry.id,
                    root,
                });
            }
        }
        return Err(format!("Context store '{}' is not registered.", id));
    }

    if let Some(path) = store_path {
        let root = PathBuf::from(path);

        // Try to read metadata id
        let metadata_path = get_context_store_metadata_path(&root);
        let id = if let Ok(content) = std::fs::read_to_string(&metadata_path) {
            match parse_metadata_state(&content) {
                Ok(metadata) => metadata.id,
                Err(_) => {
                    // Fallback to basename
                    root.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string()
                }
            }
        } else {
            // Fallback to basename
            root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        };

        return Ok(SelectedStore { id, root });
    }

    Err("Pass --store <id> or --store-path <path>.".to_string())
}

/// Find an initiative across all registered context stores.
/// Returns a Vec of (SelectedStore, InitiativeState) tuples where matches were found.
pub fn find_initiative_across_stores(
    initiative_id: &str,
    gdd: Option<&Path>,
) -> Result<Vec<(SelectedStore, InitiativeState)>, String> {
    let stores = registered_stores(gdd);
    let mut matches = Vec::new();

    // Propagate a parse/validation error (e.g. a corrupted initiative.yaml) rather than
    // silently skipping it, so a broken initiative surfaces instead of reading as "not found".
    for store in stores {
        match read_initiative(&store.root, initiative_id) {
            Ok(Some(initiative)) => matches.push((store, initiative)),
            Ok(None) => {}
            Err(e) => return Err(e),
        }
    }

    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::collections::initiatives::operations::{
        create_initiative, CreateInitiativeInput,
    };
    use crate::core::context_store::{
        save_registry, RegistryEntryState, RegistryState, BackendConfig,
    };
    use std::collections::BTreeMap;
    use tempfile::tempdir;

    #[test]
    fn test_resolve_selected_store_neither_selector() {
        let result = resolve_selected_store(None, None, None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Pass --store <id> or --store-path <path>"));
    }

    #[test]
    fn test_resolve_selected_store_both_selectors() {
        let result = resolve_selected_store(Some("team"), Some("/tmp/path"), None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Pass only one of --store or --store-path"));
    }

    #[test]
    fn test_resolve_selected_store_with_store_id() {
        let temp_dir = tempdir().unwrap();
        let gdd = Some(temp_dir.path());

        // Create a registry with a store
        let mut stores = BTreeMap::new();
        stores.insert(
            "team".to_string(),
            RegistryEntryState {
                backend: BackendConfig::Git {
                    local_path: "/tmp/team-store".to_string(),
                    remote: None,
                    branch: None,
                },
            },
        );
        let registry = RegistryState {
            version: 1,
            stores,
        };
        save_registry(&registry, gdd).unwrap();

        let result = resolve_selected_store(Some("team"), None, gdd);
        assert!(result.is_ok());
        let selected = result.unwrap();
        assert_eq!(selected.id, "team");
        assert_eq!(selected.root, PathBuf::from("/tmp/team-store"));
    }

    #[test]
    fn test_resolve_selected_store_with_nonexistent_store_id() {
        let temp_dir = tempdir().unwrap();
        let gdd = Some(temp_dir.path());

        let result = resolve_selected_store(Some("nonexistent"), None, gdd);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Context store 'nonexistent' is not registered"));
    }

    #[test]
    fn test_resolve_selected_store_with_store_path() {
        let temp_dir = tempdir().unwrap();
        let store_path = temp_dir.path();

        // Doesn't need metadata; will use basename
        let result = resolve_selected_store(None, Some(store_path.to_str().unwrap()), None);
        assert!(result.is_ok());
        let selected = result.unwrap();
        assert_eq!(selected.root, store_path);
        // id should be the basename of the temp dir (which tempdir creates with a random name)
        assert!(!selected.id.is_empty());
    }

    #[test]
    fn test_registered_stores_empty() {
        let temp_dir = tempdir().unwrap();
        let gdd = Some(temp_dir.path());

        let stores = registered_stores(gdd);
        assert_eq!(stores.len(), 0);
    }

    #[test]
    fn test_registered_stores_with_entries() {
        let temp_dir = tempdir().unwrap();
        let gdd = Some(temp_dir.path());

        // Create a registry with stores
        let mut stores_map = BTreeMap::new();
        stores_map.insert(
            "store-a".to_string(),
            RegistryEntryState {
                backend: BackendConfig::Git {
                    local_path: "/tmp/store-a".to_string(),
                    remote: None,
                    branch: None,
                },
            },
        );
        stores_map.insert(
            "store-b".to_string(),
            RegistryEntryState {
                backend: BackendConfig::Git {
                    local_path: "/tmp/store-b".to_string(),
                    remote: None,
                    branch: None,
                },
            },
        );
        let registry = RegistryState {
            version: 1,
            stores: stores_map,
        };
        save_registry(&registry, gdd).unwrap();

        let stores = registered_stores(gdd);
        assert_eq!(stores.len(), 2);
        assert_eq!(stores[0].id, "store-a");
        assert_eq!(stores[1].id, "store-b");
    }

    #[test]
    fn test_find_initiative_across_stores_no_matches() {
        let temp_dir = tempdir().unwrap();
        let gdd = Some(temp_dir.path());
        let store_dir = tempdir().unwrap();

        // Create a registry with one store
        let mut stores_map = BTreeMap::new();
        stores_map.insert(
            "team".to_string(),
            RegistryEntryState {
                backend: BackendConfig::Git {
                    local_path: store_dir.path().to_string_lossy().to_string(),
                    remote: None,
                    branch: None,
                },
            },
        );
        let registry = RegistryState {
            version: 1,
            stores: stores_map,
        };
        save_registry(&registry, gdd).unwrap();

        let result = find_initiative_across_stores("nonexistent", gdd);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_find_initiative_across_stores_with_match() {
        let temp_dir = tempdir().unwrap();
        let gdd = Some(temp_dir.path());
        let store_dir = tempdir().unwrap();

        // Create an initiative in the store
        let created = create_initiative(
            store_dir.path(),
            CreateInitiativeInput {
                id: "roadmap".to_string(),
                title: "Roadmap".to_string(),
                summary: "Project roadmap".to_string(),
                ..Default::default()
            },
        )
        .unwrap();

        // Create a registry pointing to the store
        let mut stores_map = BTreeMap::new();
        stores_map.insert(
            "team".to_string(),
            RegistryEntryState {
                backend: BackendConfig::Git {
                    local_path: store_dir.path().to_string_lossy().to_string(),
                    remote: None,
                    branch: None,
                },
            },
        );
        let registry = RegistryState {
            version: 1,
            stores: stores_map,
        };
        save_registry(&registry, gdd).unwrap();

        let result = find_initiative_across_stores("roadmap", gdd);
        assert!(result.is_ok());
        let matches = result.unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].0.id, "team");
        assert_eq!(matches[0].1.id, "roadmap");
        assert_eq!(matches[0].1.title, "Roadmap");
        assert_eq!(matches[0].1.title, created.title);
    }
}
