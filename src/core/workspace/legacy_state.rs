use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::path::Path;

use super::foundation::{
    validate_workspace_link_name, validate_workspace_name, PreferredOpener, WorkspaceContext,
    WorkspaceSkillState, WorkspaceViewState, WORKSPACE_METADATA_DIR_NAME,
};

// Constants
pub const WORKSPACE_LEGACY_SHARED_STATE_FILE_NAME: &str = "workspace.yaml";
pub const WORKSPACE_LEGACY_LOCAL_STATE_FILE_NAME: &str = "local.yaml";

// Legacy state structures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceSharedState {
    pub version: u8,
    pub name: String,
    pub context: Option<WorkspaceContext>,
    pub links: BTreeMap<String, Value>, // links map to objects (arbitrary YAML)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceLocalState {
    pub version: u8,
    pub paths: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub preferred_opener: Option<PreferredOpener>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub workspace_skills: Option<WorkspaceSkillState>,
}

// Path helpers
pub fn get_workspace_legacy_shared_state_path(workspace_root: &Path) -> std::path::PathBuf {
    workspace_root
        .join(WORKSPACE_METADATA_DIR_NAME)
        .join(WORKSPACE_LEGACY_SHARED_STATE_FILE_NAME)
}

pub fn get_workspace_legacy_local_state_path(workspace_root: &Path) -> std::path::PathBuf {
    workspace_root
        .join(WORKSPACE_METADATA_DIR_NAME)
        .join(WORKSPACE_LEGACY_LOCAL_STATE_FILE_NAME)
}

// Parse helpers
pub fn parse_workspace_shared_state(content: &str) -> Result<WorkspaceSharedState, String> {
    let raw: Value = serde_yaml::from_str(content)
        .map_err(|e| format!("Failed to parse workspace shared state: {}", e))?;

    // Try to parse as a legacy shared state (version + name + context + links of objects)
    let as_map = raw
        .as_mapping()
        .ok_or("Workspace shared state must be an object")?;

    let version_key = Value::String("version".to_string());
    let version = as_map
        .get(&version_key)
        .and_then(|v| v.as_u64())
        .ok_or("Missing or invalid version in workspace shared state")? as u8;

    if version != 1 {
        return Err(format!(
            "Workspace shared state version must be 1, got {}",
            version
        ));
    }

    let name_key = Value::String("name".to_string());
    let name = as_map
        .get(&name_key)
        .and_then(|v| v.as_str())
        .ok_or("Missing or invalid name in workspace shared state")?
        .to_string();

    validate_workspace_name(&name).map_err(|e| format!("Invalid workspace name: {}", e))?;

    let context_key = Value::String("context".to_string());
    let context = as_map.get(&context_key).cloned();

    let context_parsed = if let Some(ctx) = context {
        if ctx.is_null() {
            None
        } else {
            // Try to deserialize context from the YAML value
            serde_yaml::from_value::<WorkspaceContext>(ctx)
                .map(Some)
                .map_err(|e| format!("Failed to parse workspace context: {}", e))?
        }
    } else {
        None
    };

    let links_key = Value::String("links".to_string());
    let links_raw = as_map
        .get(&links_key)
        .and_then(|v| v.as_mapping())
        .ok_or("Missing or invalid links in workspace shared state")?;

    let mut links = BTreeMap::new();
    for (k, v) in links_raw.iter() {
        let key = k.as_str().ok_or("Link name must be a string")?.to_string();

        validate_workspace_link_name(&key)
            .map_err(|e| format!("Invalid workspace link name '{}': {}", key, e))?;

        links.insert(key, v.clone());
    }

    Ok(WorkspaceSharedState {
        version: 1,
        name,
        context: context_parsed,
        links,
    })
}

pub fn parse_workspace_local_state(content: &str) -> Result<WorkspaceLocalState, String> {
    let raw: Value = serde_yaml::from_str(content)
        .map_err(|e| format!("Failed to parse workspace local state: {}", e))?;

    let as_map = raw
        .as_mapping()
        .ok_or("Workspace local state must be an object")?;

    let version_key = Value::String("version".to_string());
    let version = as_map
        .get(&version_key)
        .and_then(|v| v.as_u64())
        .ok_or("Missing or invalid version in workspace local state")? as u8;

    if version != 1 {
        return Err(format!(
            "Workspace local state version must be 1, got {}",
            version
        ));
    }

    let paths_key = Value::String("paths".to_string());
    let paths_raw = as_map
        .get(&paths_key)
        .and_then(|v| v.as_mapping())
        .ok_or("Missing or invalid paths in workspace local state")?;

    let mut paths = BTreeMap::new();
    for (k, v) in paths_raw.iter() {
        let key = k.as_str().ok_or("Path name must be a string")?.to_string();

        validate_workspace_link_name(&key)
            .map_err(|e| format!("Invalid workspace local path name '{}': {}", key, e))?;

        let path = v
            .as_str()
            .ok_or_else(|| format!("Path value for '{}' must be a string", key))?
            .to_string();

        paths.insert(key, path);
    }

    let preferred_opener_key = Value::String("preferred_opener".to_string());
    let preferred_opener = as_map.get(&preferred_opener_key).cloned();

    let preferred_opener_parsed = if let Some(po) = preferred_opener {
        if po.is_null() {
            None
        } else {
            Some(
                serde_yaml::from_value::<PreferredOpener>(po)
                    .map_err(|e| format!("Failed to parse preferred_opener: {}", e))?,
            )
        }
    } else {
        None
    };

    let tools_key = Value::String("tools".to_string());
    let tools = as_map.get(&tools_key).cloned().and_then(|v| {
        if v.is_null() {
            None
        } else {
            serde_yaml::from_value::<Vec<String>>(v).ok()
        }
    });

    let workspace_skills_key = Value::String("workspace_skills".to_string());
    let workspace_skills = as_map.get(&workspace_skills_key).cloned().and_then(|v| {
        if v.is_null() {
            None
        } else {
            serde_yaml::from_value::<WorkspaceSkillState>(v).ok()
        }
    });

    Ok(WorkspaceLocalState {
        version: 1,
        paths,
        preferred_opener: preferred_opener_parsed,
        tools,
        workspace_skills,
    })
}

// Merge legacy state parts into view state
pub fn workspace_state_parts_to_view_state(
    shared_state: WorkspaceSharedState,
    local_state: Option<WorkspaceLocalState>,
) -> WorkspaceViewState {
    let mut link_names: Vec<String> = shared_state.links.keys().cloned().collect();
    if let Some(ref local) = local_state {
        for key in local.paths.keys() {
            if !link_names.contains(key) {
                link_names.push(key.clone());
            }
        }
    }

    link_names.sort();

    let links: BTreeMap<String, Option<String>> = link_names
        .into_iter()
        .map(|name| {
            let path = local_state
                .as_ref()
                .and_then(|local| local.paths.get(&name).cloned());
            (name, path)
        })
        .collect();

    WorkspaceViewState {
        version: 1,
        name: shared_state.name,
        context: shared_state.context,
        links,
        preferred_opener: local_state
            .as_ref()
            .and_then(|l| l.preferred_opener.clone()),
        tools: local_state.as_ref().and_then(|l| l.tools.clone()),
        workspace_skills: local_state
            .as_ref()
            .and_then(|l| l.workspace_skills.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::workspace::OpenerKind;

    #[test]
    fn test_legacy_shared_state_parse_and_merge() {
        let shared_yaml = r#"
version: 1
name: test-ws
context: null
links:
  repo: {}
"#;

        let shared = parse_workspace_shared_state(shared_yaml).expect("parse shared failed");
        assert_eq!(shared.name, "test-ws");
        assert!(shared.context.is_none());
        assert_eq!(shared.links.len(), 1);
        assert!(shared.links.contains_key("repo"));

        // Merge with no local state
        let view = workspace_state_parts_to_view_state(shared, None);
        assert_eq!(view.name, "test-ws");
        assert!(view.context.is_none());
        assert_eq!(view.links.len(), 1);
        assert_eq!(view.links.get("repo"), Some(&None));
    }

    #[test]
    fn test_legacy_shared_and_local_state_merge() {
        let shared_yaml = r#"
version: 1
name: test-ws
context: null
links:
  repo: {}
  docs: {}
"#;

        let local_yaml = r#"
version: 1
paths:
  repo: /absolute/path/to/repo
"#;

        let shared = parse_workspace_shared_state(shared_yaml).expect("parse shared failed");
        let local = parse_workspace_local_state(local_yaml).expect("parse local failed");

        let view = workspace_state_parts_to_view_state(shared, Some(local));
        assert_eq!(view.name, "test-ws");
        assert_eq!(view.links.len(), 2);
        assert_eq!(
            view.links.get("repo"),
            Some(&Some("/absolute/path/to/repo".to_string()))
        );
        assert_eq!(view.links.get("docs"), Some(&None));
    }

    #[test]
    fn test_legacy_local_state_with_preferred_opener() {
        let local_yaml = r#"
version: 1
paths: {}
preferred_opener:
  kind: agent
  id: claude
"#;

        let local = parse_workspace_local_state(local_yaml).expect("parse local failed");
        assert!(local.preferred_opener.is_some());
        let po = local.preferred_opener.unwrap();
        assert_eq!(po.kind, OpenerKind::Agent);
        assert_eq!(po.id, "claude");
    }

    #[test]
    fn test_legacy_shared_state_version_validation() {
        let yaml = r#"
version: 2
name: test-ws
context: null
links: {}
"#;
        let result = parse_workspace_shared_state(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("version must be 1"));
    }

    #[test]
    fn test_legacy_shared_state_name_validation() {
        let yaml = r#"
version: 1
name: InvalidName
context: null
links: {}
"#;
        let result = parse_workspace_shared_state(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid workspace name"));
    }
}
