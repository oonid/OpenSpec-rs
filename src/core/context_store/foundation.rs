use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::core::config::xdg_data_dir;

// Constants (must match upstream exactly)
pub const CONTEXT_STORE_METADATA_DIR_NAME: &str = ".openspec-store";
pub const CONTEXT_STORE_METADATA_FILE_NAME: &str = "store.yaml";
pub const CONTEXT_STORES_DIR_NAME: &str = "context-stores";
pub const CONTEXT_STORE_REGISTRY_FILE_NAME: &str = "registry.yaml";

// Path helpers.
//
// `global_data_dir` mirrors upstream's `ContextStorePathOptions { globalDataDir }`: pass
// `None` to use the real OpenSpec global data dir, or `Some(dir)` to point at an isolated
// directory (used by the operations/CLI layer's `--path` handling and by tests).
pub fn get_context_stores_dir(global_data_dir: Option<&Path>) -> PathBuf {
    let base = match global_data_dir {
        Some(dir) => dir.to_path_buf(),
        None => xdg_data_dir(),
    };
    base.join(CONTEXT_STORES_DIR_NAME)
}

pub fn get_context_store_registry_path(global_data_dir: Option<&Path>) -> PathBuf {
    get_context_stores_dir(global_data_dir).join(CONTEXT_STORE_REGISTRY_FILE_NAME)
}

pub fn get_default_context_store_root(id: &str, global_data_dir: Option<&Path>) -> PathBuf {
    get_context_stores_dir(global_data_dir).join(id)
}

pub fn get_context_store_metadata_dir(store_root: &Path) -> PathBuf {
    store_root.join(CONTEXT_STORE_METADATA_DIR_NAME)
}

pub fn get_context_store_metadata_path(store_root: &Path) -> PathBuf {
    get_context_store_metadata_dir(store_root).join(CONTEXT_STORE_METADATA_FILE_NAME)
}

// ID validation
pub fn is_valid_context_store_id(id: &str) -> bool {
    validate_context_store_id(id).is_ok()
}

pub fn validate_context_store_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("Context store id must not be empty".to_string());
    }

    if id == "." || id == ".." {
        return Err(format!("Context store id must not be '{}'", id));
    }

    if id.contains('/') || id.contains('\\') {
        return Err("Context store id must not contain path separators".to_string());
    }

    let regex = regex::Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
    if !regex.is_match(id) {
        return Err(
            "Context store id must be kebab-case with lowercase letters, numbers, and single hyphen separators".to_string(),
        );
    }

    Ok(())
}

// Serde structs (MUST match YAML exactly)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BackendConfig {
    Git {
        local_path: String,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        remote: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        branch: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistryEntryState {
    pub backend: BackendConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistryState {
    pub version: u8,
    #[serde(default)]
    pub stores: BTreeMap<String, RegistryEntryState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetadataState {
    pub version: u8,
    pub id: String,
}

/// Flattened registry entry (id + backend) for listing/returning.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextStoreRegistryEntry {
    pub id: String,
    pub backend: BackendConfig,
}

// Parse/serialize helpers
pub fn parse_registry_state(content: &str) -> Result<RegistryState, String> {
    let parsed: RegistryState = serde_yaml::from_str(content)
        .map_err(|e| format!("Failed to parse registry state: {}", e))?;

    // Validate all store IDs
    for id in parsed.stores.keys() {
        validate_context_store_id(id).map_err(|e| format!("Invalid store id '{}': {}", id, e))?;
    }

    Ok(parsed)
}

pub fn serialize_registry_state(state: &RegistryState) -> Result<String, String> {
    // Validate all store IDs before serializing
    for id in state.stores.keys() {
        validate_context_store_id(id).map_err(|e| format!("Invalid store id '{}': {}", id, e))?;
    }

    serde_yaml::to_string(state).map_err(|e| format!("Failed to serialize registry state: {}", e))
}

pub fn parse_metadata_state(content: &str) -> Result<MetadataState, String> {
    let parsed: MetadataState = serde_yaml::from_str(content)
        .map_err(|e| format!("Failed to parse metadata state: {}", e))?;

    validate_context_store_id(&parsed.id)
        .map_err(|e| format!("Invalid metadata id '{}': {}", parsed.id, e))?;

    Ok(parsed)
}

pub fn serialize_metadata_state(state: &MetadataState) -> Result<String, String> {
    validate_context_store_id(&state.id)
        .map_err(|e| format!("Invalid metadata id '{}': {}", state.id, e))?;

    serde_yaml::to_string(state).map_err(|e| format!("Failed to serialize metadata state: {}", e))
}

// Atomic write helper
pub fn write_file_atomically(path: &Path, content: &str) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::Write;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Create a temporary file in the same directory as the target
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name().unwrap_or_default();
    let temp_path = parent.join(format!(".{}.tmp", file_name.to_string_lossy()));

    // Write to temp file
    let mut temp_file = File::create(&temp_path)?;
    temp_file.write_all(content.as_bytes())?;
    drop(temp_file);

    // Atomically rename temp file to target
    std::fs::rename(&temp_path, path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_validation_valid() {
        assert!(is_valid_context_store_id("my-store"));
        assert!(is_valid_context_store_id("store1"));
        assert!(is_valid_context_store_id("a-b-c"));
        assert!(is_valid_context_store_id("a"));
        assert!(is_valid_context_store_id("123"));
    }

    #[test]
    fn test_id_validation_invalid() {
        assert!(!is_valid_context_store_id(""));
        assert!(!is_valid_context_store_id("."));
        assert!(!is_valid_context_store_id(".."));
        assert!(!is_valid_context_store_id("My-Store")); // uppercase
        assert!(!is_valid_context_store_id("a/b")); // slash
        assert!(!is_valid_context_store_id("a\\b")); // backslash
        assert!(!is_valid_context_store_id("a--b")); // double hyphen
        assert!(!is_valid_context_store_id("-a")); // leading hyphen
        assert!(!is_valid_context_store_id("a-")); // trailing hyphen
    }

    #[test]
    fn test_registry_round_trip_with_remote_branch() {
        let mut stores = BTreeMap::new();
        stores.insert(
            "my-store".to_string(),
            RegistryEntryState {
                backend: BackendConfig::Git {
                    local_path: "/path/to/store".to_string(),
                    remote: Some("https://example.com/repo".to_string()),
                    branch: Some("main".to_string()),
                },
            },
        );

        let state = RegistryState { version: 1, stores };

        let serialized = serialize_registry_state(&state).expect("serialize failed");
        assert!(serialized.contains("type: git"));
        assert!(serialized.contains("local_path: /path/to/store"));
        assert!(serialized.contains("remote: https://example.com/repo"));
        assert!(serialized.contains("branch: main"));

        let parsed = parse_registry_state(&serialized).expect("parse failed");
        assert_eq!(parsed, state);
    }

    #[test]
    fn test_registry_round_trip_without_remote_branch() {
        let mut stores = BTreeMap::new();
        stores.insert(
            "simple-store".to_string(),
            RegistryEntryState {
                backend: BackendConfig::Git {
                    local_path: "/path/to/simple".to_string(),
                    remote: None,
                    branch: None,
                },
            },
        );

        let state = RegistryState { version: 1, stores };

        let serialized = serialize_registry_state(&state).expect("serialize failed");
        assert!(serialized.contains("type: git"));
        assert!(serialized.contains("local_path: /path/to/simple"));
        // When None, these should not appear in serialized output
        assert!(!serialized.contains("remote:"));
        assert!(!serialized.contains("branch:"));

        let parsed = parse_registry_state(&serialized).expect("parse failed");
        assert_eq!(parsed, state);
    }

    #[test]
    fn test_metadata_round_trip() {
        let state = MetadataState {
            version: 1,
            id: "test-store".to_string(),
        };

        let serialized = serialize_metadata_state(&state).expect("serialize failed");
        let parsed = parse_metadata_state(&serialized).expect("parse failed");
        assert_eq!(parsed, state);
    }

    #[test]
    fn test_atomic_write() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let target_path = temp_dir.path().join("test.txt");

        let content1 = "first content";
        write_file_atomically(&target_path, content1).expect("first write failed");
        let read_back = std::fs::read_to_string(&target_path).expect("read failed");
        assert_eq!(read_back, content1);

        let content2 = "second content";
        write_file_atomically(&target_path, content2).expect("second write failed");
        let read_back = std::fs::read_to_string(&target_path).expect("read failed");
        assert_eq!(read_back, content2);
    }

    #[test]
    fn test_validate_context_store_id_errors() {
        let result = validate_context_store_id("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));

        let result = validate_context_store_id(".");
        assert!(result.is_err());

        let result = validate_context_store_id("..");
        assert!(result.is_err());

        let result = validate_context_store_id("a/b");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("path separators"));

        let result = validate_context_store_id("A-B");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("kebab-case"));
    }

    #[test]
    fn test_path_helpers() {
        let stores_dir = get_context_stores_dir(None);
        assert!(stores_dir.ends_with("context-stores"));

        let registry_path = get_context_store_registry_path(None);
        assert!(registry_path.ends_with("registry.yaml"));

        let store_root = get_default_context_store_root("my-store", None);
        assert!(store_root.ends_with("my-store"));

        // With an explicit global data dir override, paths are rooted under it.
        let base = std::path::Path::new("/tmp/os-test-data");
        assert_eq!(
            get_context_store_registry_path(Some(base)),
            base.join("context-stores").join("registry.yaml")
        );

        let metadata_dir = get_context_store_metadata_dir(&store_root);
        assert!(metadata_dir.ends_with(".openspec-store"));

        let metadata_path = get_context_store_metadata_path(&store_root);
        assert!(metadata_path.ends_with("store.yaml"));
    }
}
