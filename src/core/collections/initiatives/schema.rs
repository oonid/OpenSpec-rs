use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Constants (must match upstream exactly)
pub const INITIATIVE_COLLECTION_ID: &str = "initiatives";
pub const INITIATIVE_FILE_NAME: &str = "initiative.yaml";
pub const INITIATIVE_REQUIREMENTS_FILE_NAME: &str = "requirements.md";
pub const INITIATIVE_DESIGN_FILE_NAME: &str = "design.md";
pub const INITIATIVE_DECISIONS_FILE_NAME: &str = "decisions.md";
pub const INITIATIVE_QUESTIONS_FILE_NAME: &str = "questions.md";
pub const INITIATIVE_TASKS_FILE_NAME: &str = "tasks.md";

pub const INITIATIVE_MARKDOWN_FILE_NAMES: &[&str] = &[
    INITIATIVE_REQUIREMENTS_FILE_NAME,
    INITIATIVE_DESIGN_FILE_NAME,
    INITIATIVE_DECISIONS_FILE_NAME,
    INITIATIVE_QUESTIONS_FILE_NAME,
    INITIATIVE_TASKS_FILE_NAME,
];

// Status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InitiativeStatus {
    Exploring,
    Active,
    Complete,
    Archived,
}

// Initiative state struct
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitiativeState {
    pub version: u8,
    pub id: String,
    pub title: String,
    pub summary: String,
    pub status: InitiativeStatus,
    pub created: String,
    #[serde(default)]
    pub owners: Vec<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, serde_yaml::Value>,
}

/// Validate an initiative ID according to upstream rules.
/// Returns Ok(()) if valid, Err(message) if invalid.
pub fn validate_initiative_id(id: &str) -> Result<(), String> {
    // Check for NUL bytes
    if id.contains('\0') {
        return Err("Initiative id must not contain NUL bytes".to_string());
    }

    // Check for empty
    if id.is_empty() {
        return Err("Initiative id must not be empty".to_string());
    }

    // Check for . or ..
    if id == "." || id == ".." {
        return Err(format!("Initiative id must not be '{}'", id));
    }

    // Check for path separators
    if id.contains('/') || id.contains('\\') {
        return Err("Initiative id must not contain path separators".to_string());
    }

    // Check kebab-case format
    let regex = regex::Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
    if !regex.is_match(id) {
        return Err(
            "Initiative id must be kebab-case with lowercase letters, numbers, and single hyphen separators".to_string(),
        );
    }

    Ok(())
}

/// Check if an initiative ID is valid (convenience function that swallows errors).
pub fn is_valid_initiative_id(id: &str) -> bool {
    validate_initiative_id(id).is_ok()
}

/// Parse a YAML string into an InitiativeState.
/// Validates the state according to upstream rules.
pub fn parse_initiative_state(content: &str) -> Result<InitiativeState, String> {
    let raw: serde_yaml::Value = serde_yaml::from_str(content)
        .map_err(|e| format!("Invalid initiative state: {}", e))?;

    let parsed: InitiativeState = serde_yaml::from_value(raw)
        .map_err(|e| format!("Invalid initiative state: {}", e))?;

    // Validate version
    if parsed.version != 1 {
        return Err(format!(
            "version: invalid, expected version 1 but got {}",
            parsed.version
        ));
    }

    // Validate id
    validate_initiative_id(&parsed.id)
        .map_err(|e| format!("id: {}", e))?;

    // Validate title (non-blank after trim)
    if parsed.title.trim().is_empty() {
        return Err("title: must not be empty".to_string());
    }

    // Validate summary (non-blank after trim)
    if parsed.summary.trim().is_empty() {
        return Err("summary: must not be empty".to_string());
    }

    // Validate created format (YYYY-MM-DD)
    let date_regex = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    if !date_regex.is_match(&parsed.created) {
        return Err("created: must be YYYY-MM-DD format".to_string());
    }

    // Validate owners (each non-blank)
    for owner in &parsed.owners {
        if owner.trim().is_empty() {
            return Err("owners: each owner must not be empty".to_string());
        }
    }

    Ok(parsed)
}

/// Serialize an InitiativeState to YAML.
/// Normalizes the state by round-tripping through parse to ensure validation.
pub fn serialize_initiative_state(state: &InitiativeState) -> Result<String, String> {
    // Validate id first
    validate_initiative_id(&state.id)
        .map_err(|e| format!("id: {}", e))?;

    // Normalize by serializing then parsing to ensure all validation runs
    let normalized = InitiativeState {
        version: 1,
        id: state.id.clone(),
        title: state.title.clone(),
        summary: state.summary.clone(),
        status: state.status,
        created: state.created.clone(),
        owners: state.owners.clone(),
        metadata: state.metadata.clone(),
    };

    // Parse to validate
    let yaml_str = serde_yaml::to_string(&normalized)
        .map_err(|e| format!("Failed to serialize state: {}", e))?;

    parse_initiative_state(&yaml_str)?;

    Ok(yaml_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_initiative_id() {
        assert!(is_valid_initiative_id("my-init"));
        assert!(is_valid_initiative_id("a"));
        assert!(is_valid_initiative_id("123"));
        assert!(is_valid_initiative_id("a-b-c"));
    }

    #[test]
    fn test_invalid_initiative_id_empty() {
        assert!(!is_valid_initiative_id(""));
        let err = validate_initiative_id("").unwrap_err();
        assert!(err.contains("must not be empty"));
    }

    #[test]
    fn test_invalid_initiative_id_dot() {
        assert!(!is_valid_initiative_id("."));
        let err = validate_initiative_id(".").unwrap_err();
        assert!(err.contains("must not be '.'"));
    }

    #[test]
    fn test_invalid_initiative_id_double_dot() {
        assert!(!is_valid_initiative_id(".."));
        let err = validate_initiative_id("..").unwrap_err();
        assert!(err.contains("must not be '..'"));
    }

    #[test]
    fn test_invalid_initiative_id_uppercase() {
        assert!(!is_valid_initiative_id("A-B"));
        let err = validate_initiative_id("A-B").unwrap_err();
        assert!(err.contains("kebab-case"));
    }

    #[test]
    fn test_invalid_initiative_id_path_separators() {
        assert!(!is_valid_initiative_id("a/b"));
        let err = validate_initiative_id("a/b").unwrap_err();
        assert!(err.contains("path separators"));
    }

    #[test]
    fn test_invalid_initiative_id_nul_byte() {
        let id = "a\0b";
        assert!(!is_valid_initiative_id(id));
        let err = validate_initiative_id(id).unwrap_err();
        assert!(err.contains("NUL bytes"));
    }

    #[test]
    fn test_parse_valid_initiative_state() {
        let yaml = r#"version: 1
id: my-init
title: "Test Initiative"
summary: "A test summary"
status: exploring
created: "2024-01-15"
owners: []
metadata: {}
"#;
        let state = parse_initiative_state(yaml).unwrap();
        assert_eq!(state.id, "my-init");
        assert_eq!(state.title, "Test Initiative");
        assert_eq!(state.status, InitiativeStatus::Exploring);
    }

    #[test]
    fn test_parse_rejects_blank_title() {
        let yaml = r#"version: 1
id: my-init
title: "   "
summary: "A test summary"
status: exploring
created: "2024-01-15"
"#;
        let err = parse_initiative_state(yaml).unwrap_err();
        assert!(err.contains("title"));
    }

    #[test]
    fn test_parse_rejects_blank_summary() {
        let yaml = r#"version: 1
id: my-init
title: "Test"
summary: "  "
status: exploring
created: "2024-01-15"
"#;
        let err = parse_initiative_state(yaml).unwrap_err();
        assert!(err.contains("summary"));
    }

    #[test]
    fn test_parse_rejects_bad_created_format() {
        let yaml = r#"version: 1
id: my-init
title: "Test"
summary: "Summary"
status: exploring
created: "2024/01/15"
"#;
        let err = parse_initiative_state(yaml).unwrap_err();
        assert!(err.contains("created"));
        assert!(err.contains("YYYY-MM-DD"));
    }

    #[test]
    fn test_serialize_and_parse_roundtrip() {
        let state = InitiativeState {
            version: 1,
            id: "test-id".to_string(),
            title: "Test Title".to_string(),
            summary: "Test Summary".to_string(),
            status: InitiativeStatus::Active,
            created: "2024-01-15".to_string(),
            owners: vec!["owner1".to_string()],
            metadata: BTreeMap::new(),
        };

        let yaml = serialize_initiative_state(&state).unwrap();
        let parsed = parse_initiative_state(&yaml).unwrap();

        assert_eq!(parsed, state);
    }

    #[test]
    fn test_yaml_field_order() {
        let state = InitiativeState {
            version: 1,
            id: "test-id".to_string(),
            title: "Title".to_string(),
            summary: "Summary".to_string(),
            status: InitiativeStatus::Exploring,
            created: "2024-01-15".to_string(),
            owners: vec![],
            metadata: BTreeMap::new(),
        };

        let yaml = serialize_initiative_state(&state).unwrap();

        // Check that fields appear in the correct order
        let version_pos = yaml.find("version").unwrap();
        let id_pos = yaml.find("id").unwrap();
        let title_pos = yaml.find("title").unwrap();
        let summary_pos = yaml.find("summary").unwrap();
        let status_pos = yaml.find("status").unwrap();
        let created_pos = yaml.find("created").unwrap();
        let owners_pos = yaml.find("owners").unwrap();
        let metadata_pos = yaml.find("metadata").unwrap();

        assert!(version_pos < id_pos);
        assert!(id_pos < title_pos);
        assert!(title_pos < summary_pos);
        assert!(summary_pos < status_pos);
        assert!(status_pos < created_pos);
        assert!(created_pos < owners_pos);
        assert!(owners_pos < metadata_pos);
    }

    #[test]
    fn test_yaml_contains_status_lowercase() {
        let state = InitiativeState {
            version: 1,
            id: "test-id".to_string(),
            title: "Title".to_string(),
            summary: "Summary".to_string(),
            status: InitiativeStatus::Exploring,
            created: "2024-01-15".to_string(),
            owners: vec![],
            metadata: BTreeMap::new(),
        };

        let yaml = serialize_initiative_state(&state).unwrap();
        assert!(yaml.contains("status: exploring"));
    }

    #[test]
    fn test_yaml_output_format() {
        let state = InitiativeState {
            version: 1,
            id: "test-id".to_string(),
            title: "Test Title".to_string(),
            summary: "Test Summary".to_string(),
            status: InitiativeStatus::Active,
            created: "2024-01-15".to_string(),
            owners: vec!["owner1".to_string()],
            metadata: BTreeMap::new(),
        };

        let yaml = serialize_initiative_state(&state).unwrap();
        println!("YAML output:\n{}", yaml);

        // Verify expected content
        assert!(yaml.contains("version: 1"));
        assert!(yaml.contains("id: test-id"));
        assert!(yaml.contains("title: Test Title"));
        assert!(yaml.contains("summary: Test Summary"));
        assert!(yaml.contains("status: active"));
        assert!(yaml.contains("created: 2024-01-15"));
        assert!(yaml.contains("owners:"));
        assert!(yaml.contains("- owner1"));
    }
}
