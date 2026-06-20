use chrono::Utc;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

use super::schema::{
    InitiativeState, InitiativeStatus, INITIATIVE_FILE_NAME,
    parse_initiative_state, serialize_initiative_state, validate_initiative_id,
};
use super::templates::build_default_initiative_files;

/// Input parameters for creating a new initiative.
#[derive(Debug, Clone, Default)]
pub struct CreateInitiativeInput {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub status: Option<InitiativeStatus>,
    pub owners: Vec<String>,
    pub metadata: BTreeMap<String, serde_yaml::Value>,
    pub created: Option<String>,
}

/// Get the initiatives directory for a given store root.
pub fn initiatives_dir(store_root: &Path) -> PathBuf {
    store_root.join("initiatives")
}

/// Create a new initiative.
///
/// Parameters:
/// - store_root: the root directory of the context store
/// - input: the initiative input containing id, title, summary, status, owners, metadata, and created
///
/// Returns the created InitiativeState or an error.
pub fn create_initiative(
    store_root: &Path,
    input: CreateInitiativeInput,
) -> Result<InitiativeState, String> {
    // Validate id first
    validate_initiative_id(&input.id)?;

    // Normalize state by round-tripping through serialize+parse
    let normalized_state = InitiativeState {
        version: 1,
        id: input.id.clone(),
        title: input.title.clone(),
        summary: input.summary.clone(),
        status: input.status.unwrap_or(InitiativeStatus::Exploring),
        created: input.created.unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string()),
        owners: input.owners.clone(),
        metadata: input.metadata.clone(),
    };

    // Serialize and parse to validate
    let yaml_content = serialize_initiative_state(&normalized_state)?;
    let state = parse_initiative_state(&yaml_content)?;

    let initiatives_root = initiatives_dir(store_root);
    let initiative_dir = initiatives_root.join(&state.id);

    // Create initiatives directory (recursive)
    fs::create_dir_all(&initiatives_root)
        .map_err(|e| format!("Failed to create initiatives directory: {}", e))?;

    // Create initiative-specific directory (non-recursive)
    if let Err(e) = fs::create_dir(&initiative_dir) {
        if e.kind() == io::ErrorKind::AlreadyExists {
            return Err(format!(
                "Initiative '{}' already exists at {}",
                state.id,
                initiative_dir.display()
            ));
        }
        return Err(format!("Failed to create initiative directory: {}", e));
    }

    // Write initiative.yaml
    let yaml_path = initiative_dir.join(INITIATIVE_FILE_NAME);
    if let Err(e) = write_exclusive(&yaml_path, &yaml_content) {
        let _ = cleanup_initiative_dir(&initiative_dir, &state.id);
        return Err(format!("Failed to write initiative.yaml: {}", e));
    }

    // Write template files
    let template_files = build_default_initiative_files(&state);
    for template_file in template_files {
        let file_path = initiative_dir.join(&template_file.file_name);
        if let Err(e) = write_exclusive(&file_path, &template_file.content) {
            let _ = cleanup_initiative_dir(&initiative_dir, &state.id);
            return Err(format!(
                "Failed to write {}: {}",
                template_file.file_name, e
            ));
        }
    }

    Ok(state)
}

/// Read an initiative from disk.
/// Returns Ok(Some(state)) if found and valid, Ok(None) if not found, or Err if invalid.
pub fn read_initiative(store_root: &Path, id: &str) -> Result<Option<InitiativeState>, String> {
    validate_initiative_id(id)?;
    let initiatives_root = initiatives_dir(store_root);
    let initiative_dir = initiatives_root.join(id);
    let yaml_path = initiative_dir.join(INITIATIVE_FILE_NAME);

    // Read the file
    let content = match fs::read_to_string(&yaml_path) {
        Ok(c) => c,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(format!(
                "Failed to read initiative '{}': {}",
                id, e
            ))
        }
    };

    // Parse and validate
    let state = parse_initiative_state(&content)?;

    // Verify id matches folder name
    if state.id != id {
        return Err(format!(
            "Invalid initiative '{}': initiative.yaml id '{}' must match folder name",
            id, state.id
        ));
    }

    Ok(Some(state))
}

/// List all initiatives in the store.
/// Returns an empty vec if the initiatives directory doesn't exist.
pub fn list_initiatives(store_root: &Path) -> Result<Vec<InitiativeState>, String> {
    let initiatives_root = initiatives_dir(store_root);

    // Try to read the initiatives directory
    let entries = match fs::read_dir(&initiatives_root) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => {
            return Err(format!(
                "Failed to list initiatives directory: {}",
                e
            ))
        }
    };

    let mut initiatives = Vec::new();

    for entry_result in entries {
        let entry = entry_result.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        // Skip non-directories
        if !path.is_dir() {
            continue;
        }

        let dir_name = entry
            .file_name()
            .into_string()
            .map_err(|_| "Invalid directory name (non-UTF8)".to_string())?;

        let yaml_path = path.join(INITIATIVE_FILE_NAME);

        // Read the YAML file
        let content = match fs::read_to_string(&yaml_path) {
            Ok(c) => c,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // Skip if initiative.yaml is missing
                continue;
            }
            Err(e) => {
                return Err(format!(
                    "Invalid initiative '{}': failed to read {}: {}",
                    dir_name, INITIATIVE_FILE_NAME, e
                ))
            }
        };

        // Parse and validate
        let state = parse_initiative_state(&content)?;

        // Verify id matches folder name
        if state.id != dir_name {
            return Err(format!(
                "Invalid initiative '{}': {} id '{}' must match folder name",
                dir_name, INITIATIVE_FILE_NAME, state.id
            ));
        }

        initiatives.push(state);
    }

    // Sort by id
    initiatives.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(initiatives)
}

/// Write a file exclusively (fail if it already exists).
fn write_exclusive(path: &Path, content: &str) -> io::Result<()> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(content.as_bytes())?;
    Ok(())
}

/// Clean up an initiative directory on error.
fn cleanup_initiative_dir(dir: &Path, id: &str) -> Result<(), String> {
    fs::remove_dir_all(dir)
        .map_err(|e| format!("Failed to cleanup initiative '{}' directory: {}", id, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_initiative_creates_files() {
        let dir = tempdir().unwrap();
        let store_root = dir.path();

        let state = create_initiative(
            store_root,
            CreateInitiativeInput {
                id: "test-init".into(),
                title: "Test Initiative".into(),
                summary: "A test initiative".into(),
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(state.id, "test-init");
        assert_eq!(state.title, "Test Initiative");
        assert_eq!(state.status, InitiativeStatus::Exploring);

        // Check that files exist
        let init_dir = initiatives_dir(store_root).join("test-init");
        assert!(init_dir.join("initiative.yaml").exists());
        assert!(init_dir.join("requirements.md").exists());
        assert!(init_dir.join("design.md").exists());
        assert!(init_dir.join("decisions.md").exists());
        assert!(init_dir.join("questions.md").exists());
        assert!(init_dir.join("tasks.md").exists());
    }

    #[test]
    fn test_create_initiative_yaml_roundtrips() {
        let dir = tempdir().unwrap();
        let store_root = dir.path();

        let created_state = create_initiative(
            store_root,
            CreateInitiativeInput {
                id: "test-init".into(),
                title: "Test Initiative".into(),
                summary: "A test initiative".into(),
                status: Some(InitiativeStatus::Active),
                owners: vec!["owner1".to_string()],
                metadata: BTreeMap::new(),
                created: Some("2024-01-15".to_string()),
            },
        )
        .unwrap();

        // Read it back
        let read_state = read_initiative(store_root, "test-init")
            .unwrap()
            .unwrap();

        assert_eq!(created_state, read_state);
    }

    #[test]
    fn test_create_initiative_duplicate_id_fails() {
        let dir = tempdir().unwrap();
        let store_root = dir.path();

        create_initiative(
            store_root,
            CreateInitiativeInput {
                id: "test-init".into(),
                title: "First".into(),
                summary: "Summary".into(),
                ..Default::default()
            },
        )
        .unwrap();

        let result = create_initiative(
            store_root,
            CreateInitiativeInput {
                id: "test-init".into(),
                title: "Second".into(),
                summary: "Another summary".into(),
                ..Default::default()
            },
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_read_initiative_missing_returns_none() {
        let dir = tempdir().unwrap();
        let store_root = dir.path();

        let result = read_initiative(store_root, "nonexistent").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_read_initiative_id_mismatch_errors() {
        let dir = tempdir().unwrap();
        let store_root = dir.path();

        // Create an initiative
        create_initiative(
            store_root,
            CreateInitiativeInput {
                id: "test-init".into(),
                title: "Test".into(),
                summary: "Summary".into(),
                ..Default::default()
            },
        )
        .unwrap();

        // Manually edit the YAML to have a different id
        let yaml_path = initiatives_dir(store_root)
            .join("test-init")
            .join("initiative.yaml");
        let content = fs::read_to_string(&yaml_path).unwrap();
        let modified = content.replace("id: test-init", "id: different-id");
        fs::write(&yaml_path, modified).unwrap();

        // Try to read - should error
        let result = read_initiative(store_root, "test-init");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must match folder name"));
    }

    #[test]
    fn test_list_initiatives_empty_dir() {
        let dir = tempdir().unwrap();
        let store_root = dir.path();

        let initiatives = list_initiatives(store_root).unwrap();
        assert_eq!(initiatives.len(), 0);
    }

    #[test]
    fn test_list_initiatives_multiple() {
        let dir = tempdir().unwrap();
        let store_root = dir.path();

        create_initiative(
            store_root,
            CreateInitiativeInput {
                id: "zebra-init".into(),
                title: "Zebra".into(),
                summary: "Summary".into(),
                ..Default::default()
            },
        )
        .unwrap();

        create_initiative(
            store_root,
            CreateInitiativeInput {
                id: "alpha-init".into(),
                title: "Alpha".into(),
                summary: "Summary".into(),
                ..Default::default()
            },
        )
        .unwrap();

        let initiatives = list_initiatives(store_root).unwrap();
        assert_eq!(initiatives.len(), 2);
        assert_eq!(initiatives[0].id, "alpha-init");
        assert_eq!(initiatives[1].id, "zebra-init");
    }

    #[test]
    fn test_list_initiatives_skips_missing_yaml() {
        let dir = tempdir().unwrap();
        let store_root = dir.path();

        create_initiative(
            store_root,
            CreateInitiativeInput {
                id: "valid-init".into(),
                title: "Valid".into(),
                summary: "Summary".into(),
                ..Default::default()
            },
        )
        .unwrap();

        // Create a directory without initiative.yaml
        fs::create_dir(initiatives_dir(store_root).join("orphan-dir")).unwrap();

        let initiatives = list_initiatives(store_root).unwrap();
        assert_eq!(initiatives.len(), 1);
        assert_eq!(initiatives[0].id, "valid-init");
    }

    #[test]
    fn test_list_initiatives_id_mismatch_errors() {
        let dir = tempdir().unwrap();
        let store_root = dir.path();

        create_initiative(
            store_root,
            CreateInitiativeInput {
                id: "test-init".into(),
                title: "Test".into(),
                summary: "Summary".into(),
                ..Default::default()
            },
        )
        .unwrap();

        // Manually edit the YAML to have a different id
        let yaml_path = initiatives_dir(store_root)
            .join("test-init")
            .join("initiative.yaml");
        let content = fs::read_to_string(&yaml_path).unwrap();
        let modified = content.replace("id: test-init", "id: different-id");
        fs::write(&yaml_path, modified).unwrap();

        // Try to list - should error
        let result = list_initiatives(store_root);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must match folder name"));
    }

    #[test]
    fn test_create_with_explicit_status() {
        let dir = tempdir().unwrap();
        let store_root = dir.path();

        let state = create_initiative(
            store_root,
            CreateInitiativeInput {
                id: "test-init".into(),
                title: "Test".into(),
                summary: "Summary".into(),
                status: Some(InitiativeStatus::Complete),
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(state.status, InitiativeStatus::Complete);
    }

    #[test]
    fn test_create_with_owners() {
        let dir = tempdir().unwrap();
        let store_root = dir.path();

        let state = create_initiative(
            store_root,
            CreateInitiativeInput {
                id: "test-init".into(),
                title: "Test".into(),
                summary: "Summary".into(),
                owners: vec!["owner1".to_string(), "owner2".to_string()],
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(state.owners, vec!["owner1", "owner2"]);
    }

    #[test]
    fn test_create_generated_date_format() {
        let dir = tempdir().unwrap();
        let store_root = dir.path();

        let state = create_initiative(
            store_root,
            CreateInitiativeInput {
                id: "test-init".into(),
                title: "Test".into(),
                summary: "Summary".into(),
                ..Default::default()
            },
        )
        .unwrap();

        // Check that created is in YYYY-MM-DD format
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$")
            .unwrap()
            .is_match(&state.created));
    }
}
