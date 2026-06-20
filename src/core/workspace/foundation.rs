use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// Constants (must match upstream exactly)
pub const WORKSPACE_METADATA_DIR_NAME: &str = ".openspec-workspace";
pub const WORKSPACE_VIEW_STATE_FILE_NAME: &str = "view.yaml";
pub const WORKSPACE_CHANGES_DIR_NAME: &str = "changes";
pub const WORKSPACE_CODE_WORKSPACE_EXTENSION: &str = ".code-workspace";

// Validation helpers
fn validate_folder_style_name(name: &str, label: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err(format!("{} must not be empty", label));
    }

    if name == "." || name == ".." {
        return Err(format!("{} must not be '{}'", label, name));
    }

    if name.contains('/') || name.contains('\\') {
        return Err(format!("{} must not contain path separators", label));
    }

    Ok(())
}

pub fn validate_workspace_name(name: &str) -> Result<(), String> {
    validate_folder_style_name(name, "Workspace name")?;

    let regex = regex::Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
    if !regex.is_match(name) {
        return Err("Workspace name must be kebab-case with lowercase letters, numbers, and single hyphen separators".to_string());
    }

    Ok(())
}

pub fn validate_workspace_link_name(name: &str) -> Result<(), String> {
    validate_folder_style_name(name, "Workspace link name")
}

pub fn is_valid_workspace_name(name: &str) -> bool {
    validate_workspace_name(name).is_ok()
}

pub fn is_valid_workspace_link_name(name: &str) -> bool {
    validate_workspace_link_name(name).is_ok()
}

// Path helpers
pub fn get_workspace_metadata_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join(WORKSPACE_METADATA_DIR_NAME)
}

pub fn get_workspace_view_state_path(workspace_root: &Path) -> PathBuf {
    get_workspace_metadata_dir(workspace_root).join(WORKSPACE_VIEW_STATE_FILE_NAME)
}

pub fn get_workspace_changes_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join(WORKSPACE_CHANGES_DIR_NAME)
}

// Serde types for view.yaml and context (mirroring upstream schema exactly)

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum ContextStoreSelector {
    Registry { id: String },
    Path {
        path: String,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        observed_id: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextStoreBinding {
    pub id: String,
    pub selector: ContextStoreSelector,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceInitiativeRef {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum WorkspaceContext {
    Initiative {
        store: ContextStoreBinding,
        initiative: WorkspaceInitiativeRef,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OpenerKind {
    Agent,
    Editor,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferredOpener {
    pub kind: OpenerKind,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceSkillState {
    pub selected_agents: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub last_applied_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub last_applied_delivery: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub last_applied_workflow_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub last_applied_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceViewState {
    pub version: u8,
    pub name: String,
    pub context: Option<WorkspaceContext>,
    pub links: BTreeMap<String, Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub preferred_opener: Option<PreferredOpener>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub workspace_skills: Option<WorkspaceSkillState>,
}

// Parse/serialize helpers
pub fn parse_workspace_view_state(content: &str) -> Result<WorkspaceViewState, String> {
    let parsed: WorkspaceViewState = serde_yaml::from_str(content)
        .map_err(|e| format!("Failed to parse workspace view state: {}", e))?;

    // Validate version == 1
    if parsed.version != 1 {
        return Err(format!(
            "Workspace view state version must be 1, got {}",
            parsed.version
        ));
    }

    // Validate workspace name
    validate_workspace_name(&parsed.name)
        .map_err(|e| format!("Invalid workspace name: {}", e))?;

    // Validate all link names
    for link_name in parsed.links.keys() {
        validate_workspace_link_name(link_name)
            .map_err(|e| format!("Invalid workspace link name '{}': {}", link_name, e))?;
    }

    Ok(parsed)
}

pub fn serialize_workspace_view_state(state: &WorkspaceViewState) -> Result<String, String> {
    // Validate workspace name
    validate_workspace_name(&state.name)
        .map_err(|e| format!("Invalid workspace name: {}", e))?;

    // Validate all link names and that values are strings or null
    for link_name in state.links.keys() {
        validate_workspace_link_name(link_name)
            .map_err(|e| format!("Invalid workspace link name '{}': {}", link_name, e))?;
    }

    serde_yaml::to_string(state).map_err(|e| format!("Failed to serialize workspace view state: {}", e))
}

// Atomic write helper (matches context_store::write_file_atomically)
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
    fn test_workspace_name_validation_valid() {
        assert!(is_valid_workspace_name("my-ws"));
        assert!(is_valid_workspace_name("ws1"));
        assert!(is_valid_workspace_name("a-b-c"));
        assert!(is_valid_workspace_name("a"));
        assert!(is_valid_workspace_name("123"));
    }

    #[test]
    fn test_workspace_name_validation_invalid() {
        assert!(!is_valid_workspace_name(""));
        assert!(!is_valid_workspace_name("."));
        assert!(!is_valid_workspace_name(".."));
        assert!(!is_valid_workspace_name("My-WS")); // uppercase
        assert!(!is_valid_workspace_name("a/b")); // slash
        assert!(!is_valid_workspace_name("a\\b")); // backslash
        assert!(!is_valid_workspace_name("a--b")); // double hyphen
        assert!(!is_valid_workspace_name("-a")); // leading hyphen
        assert!(!is_valid_workspace_name("a-")); // trailing hyphen
    }

    #[test]
    fn test_link_name_validation_valid() {
        assert!(is_valid_workspace_link_name("repo_one"));
        assert!(is_valid_workspace_link_name("Repo"));
        assert!(is_valid_workspace_link_name("a.b"));
        assert!(is_valid_workspace_link_name("a"));
        assert!(is_valid_workspace_link_name("123"));
    }

    #[test]
    fn test_link_name_validation_invalid() {
        assert!(!is_valid_workspace_link_name(""));
        assert!(!is_valid_workspace_link_name("."));
        assert!(!is_valid_workspace_link_name(".."));
        assert!(!is_valid_workspace_link_name("a/b")); // slash
        assert!(!is_valid_workspace_link_name("a\\b")); // backslash
    }

    #[test]
    fn test_view_state_round_trip_minimal() {
        let state = WorkspaceViewState {
            version: 1,
            name: "my-ws".to_string(),
            context: None,
            links: BTreeMap::new(),
            preferred_opener: None,
            tools: None,
            workspace_skills: None,
        };

        let serialized = serialize_workspace_view_state(&state).expect("serialize failed");
        assert!(serialized.contains("version: 1"));
        assert!(serialized.contains("name: my-ws"));
        assert!(serialized.contains("context:"));

        let parsed = parse_workspace_view_state(&serialized).expect("parse failed");
        assert_eq!(parsed, state);
    }

    #[test]
    fn test_view_state_with_context_and_links() {
        let mut links = BTreeMap::new();
        links.insert("repo".to_string(), Some("/abs/path".to_string()));
        links.insert("docs".to_string(), None);

        let state = WorkspaceViewState {
            version: 1,
            name: "test-ws".to_string(),
            context: Some(WorkspaceContext::Initiative {
                store: ContextStoreBinding {
                    id: "store-1".to_string(),
                    selector: ContextStoreSelector::Registry {
                        id: "my-store".to_string(),
                    },
                },
                initiative: WorkspaceInitiativeRef {
                    id: "init-1".to_string(),
                },
            }),
            links,
            preferred_opener: None,
            tools: None,
            workspace_skills: None,
        };

        let serialized = serialize_workspace_view_state(&state).expect("serialize failed");
        assert!(serialized.contains("name: test-ws"));
        assert!(serialized.contains("kind: initiative"));
        assert!(serialized.contains("repo: /abs/path"));
        assert!(serialized.contains("docs:"));

        let parsed = parse_workspace_view_state(&serialized).expect("parse failed");
        assert_eq!(parsed, state);
    }

    #[test]
    fn test_view_state_version_validation() {
        let yaml = "version: 2\nname: my-ws\ncontext: null\nlinks: {}";
        let result = parse_workspace_view_state(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("version must be 1"));
    }

    #[test]
    fn test_view_state_name_validation() {
        let yaml = "version: 1\nname: InvalidName\ncontext: null\nlinks: {}";
        let result = parse_workspace_view_state(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid workspace name"));
    }

    #[test]
    fn test_view_state_link_name_validation() {
        let yaml = "version: 1\nname: my-ws\ncontext: null\nlinks:\n  'a/b': /path";
        let result = parse_workspace_view_state(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid workspace link name"));
    }
}
