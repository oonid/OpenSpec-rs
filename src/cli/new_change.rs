use crate::core::error::{OpenSpecError, Result};
use crate::core::schema::ChangeMetadata;
use regex::Regex;
use serde::Serialize;

pub const DEFAULT_SCHEMA: &str = "spec-driven";

#[derive(Debug, Serialize)]
struct ChangeInfo {
    id: String,
    path: String,
    #[serde(rename = "metadataPath")]
    metadata_path: String,
    schema: String,
}

#[derive(Debug, Serialize)]
struct NewChangeOutput {
    change: ChangeInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    initiative: Option<crate::core::schema::InitiativeLink>,
}

pub struct NewChangeOptions<'a> {
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub schema: Option<&'a str>,
    pub goal: Option<&'a str>,
    pub affected_areas: Option<&'a str>,
    pub initiative: Option<&'a str>,
    pub store: Option<&'a str>,
    pub store_path: Option<&'a str>,
    pub json: bool,
}

pub struct ValidationResult {
    pub valid: bool,
    pub error: Option<String>,
}

pub fn validate_change_name(name: &str) -> ValidationResult {
    let kebab_case_pattern = Regex::new(r"^[a-z][a-z0-9]*(-[a-z0-9]+)*$").unwrap();

    if name.is_empty() {
        return ValidationResult {
            valid: false,
            error: Some("Change name cannot be empty".to_string()),
        };
    }

    if !kebab_case_pattern.is_match(name) {
        if name.chars().any(|c| c.is_uppercase()) {
            return ValidationResult {
                valid: false,
                error: Some("Change name must be lowercase (use kebab-case)".to_string()),
            };
        }
        if name.contains(' ') {
            return ValidationResult {
                valid: false,
                error: Some("Change name cannot contain spaces (use hyphens instead)".to_string()),
            };
        }
        if name.contains('_') {
            return ValidationResult {
                valid: false,
                error: Some(
                    "Change name cannot contain underscores (use hyphens instead)".to_string(),
                ),
            };
        }
        if name.starts_with('-') {
            return ValidationResult {
                valid: false,
                error: Some("Change name cannot start with a hyphen".to_string()),
            };
        }
        if name.ends_with('-') {
            return ValidationResult {
                valid: false,
                error: Some("Change name cannot end with a hyphen".to_string()),
            };
        }
        if name.contains("--") {
            return ValidationResult {
                valid: false,
                error: Some("Change name cannot contain consecutive hyphens".to_string()),
            };
        }
        if name.chars().any(|c| !c.is_alphanumeric() && c != '-') {
            return ValidationResult {
                valid: false,
                error: Some(
                    "Change name can only contain lowercase letters, numbers, and hyphens"
                        .to_string(),
                ),
            };
        }
        if name.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
            return ValidationResult {
                valid: false,
                error: Some("Change name must start with a letter".to_string()),
            };
        }

        return ValidationResult {
            valid: false,
            error: Some(
                "Change name must follow kebab-case convention (e.g., add-auth, refactor-db)"
                    .to_string(),
            ),
        };
    }

    ValidationResult {
        valid: true,
        error: None,
    }
}

pub fn run_new_change(options: NewChangeOptions<'_>) -> Result<()> {
    let NewChangeOptions {
        name,
        description,
        schema,
        goal,
        affected_areas,
        initiative,
        store,
        store_path,
        json,
    } = options;

    let validation = validate_change_name(name);
    if !validation.valid {
        return Err(OpenSpecError::Custom(
            validation
                .error
                .unwrap_or_else(|| "Invalid change name".to_string()),
        ));
    }

    if initiative.is_none() && (store.is_some() || store_path.is_some()) {
        return Err(OpenSpecError::Custom(
            "Pass --initiative when using --store or --store-path.".to_string(),
        ));
    }

    let resolved_initiative = match initiative {
        Some(id) if id.trim().is_empty() => {
            return Err(OpenSpecError::Custom(
                "Pass --initiative <id> to link a change to an initiative.".to_string(),
            ));
        }
        Some(id) => Some(
            crate::cli::set::resolve_initiative_link(id.trim(), store, store_path)
                .map_err(OpenSpecError::Custom)?,
        ),
        None => None,
    };

    let project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    let change_dir = project_root.join("openspec").join("changes").join(name);

    if change_dir.exists() {
        return Err(OpenSpecError::Custom(format!(
            "Change '{}' already exists at {}",
            name,
            change_dir.display()
        )));
    }

    let schema_name = match schema {
        Some(s) => s.to_string(),
        None => {
            let config_manager = crate::core::config::ConfigManager::new();
            match config_manager.load_project_config() {
                Ok(config) => config.schema,
                Err(_) => DEFAULT_SCHEMA.to_string(),
            }
        }
    };

    std::fs::create_dir_all(&change_dir)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to create change directory: {}", e)))?;

    let metadata = ChangeMetadata {
        schema: schema_name.clone(),
        created: Some(chrono_lite_today()),
        goal: normalize_optional(goal),
        affected_areas: parse_affected_areas(affected_areas),
        initiative: resolved_initiative,
    };
    let metadata_path = change_dir.join(".openspec.yaml");
    let metadata_content = serde_yaml::to_string(&metadata)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to serialize metadata file: {}", e)))?;
    std::fs::write(&metadata_path, metadata_content)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to write metadata file: {}", e)))?;

    if let Some(desc) = description {
        let readme_content = format!("# {}\n\n{}\n", name, desc);
        let readme_path = change_dir.join("README.md");
        std::fs::write(&readme_path, readme_content)
            .map_err(|e| OpenSpecError::Custom(format!("Failed to write README file: {}", e)))?;
    }

    if json {
        let output = NewChangeOutput {
            change: ChangeInfo {
                id: name.to_string(),
                path: format!("openspec/changes/{}", name),
                metadata_path: format!("openspec/changes/{}/.openspec.yaml", name),
                schema: schema_name.clone(),
            },
            initiative: metadata.initiative.clone(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!(
            "Created change '{}' at openspec/changes/{}/ (schema: {})",
            name, name, schema_name
        );
    }

    Ok(())
}

fn normalize_optional(value: Option<&str>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn parse_affected_areas(value: Option<&str>) -> Option<Vec<String>> {
    let areas: Vec<String> = value
        .unwrap_or("")
        .split(',')
        .map(|area| area.trim())
        .filter(|area| !area.is_empty())
        .map(|area| area.to_string())
        .collect();

    if areas.is_empty() {
        None
    } else {
        Some(areas)
    }
}

fn chrono_lite_today() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let total_secs = now.as_secs();
    let days_since_epoch = total_secs / 86400;

    let (year, month, day) = days_to_ymd(days_since_epoch as i32);

    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn days_to_ymd(days: i32) -> (i32, u32, u32) {
    let mut year = 1970;
    let mut remaining_days = days;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    let mut day = 1u32;

    for &days_in_month in &days_in_months {
        if remaining_days < days_in_month {
            day = (remaining_days + 1) as u32;
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }

    (year, month, day)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_affected_areas_trims_and_ignores_empty_values() {
        assert_eq!(parse_affected_areas(None), None);
        assert_eq!(parse_affected_areas(Some("")), None);
        assert_eq!(
            parse_affected_areas(Some(" cli , docs ,, ")),
            Some(vec!["cli".to_string(), "docs".to_string()])
        );
    }

    #[test]
    fn test_build_json_output_uses_id_and_relative_path() {
        let output = NewChangeOutput {
            change: ChangeInfo {
                id: "add-telemetry".to_string(),
                path: format!("openspec/changes/{}", "add-telemetry"),
                metadata_path: "openspec/changes/add-telemetry/.openspec.yaml".to_string(),
                schema: "spec-driven".to_string(),
            },
            initiative: None,
        };

        let value = serde_json::to_value(&output).unwrap();
        assert_eq!(value["change"]["id"], "add-telemetry");
        assert_eq!(value["change"]["path"], "openspec/changes/add-telemetry");
        assert_eq!(
            value["change"]["metadataPath"],
            "openspec/changes/add-telemetry/.openspec.yaml"
        );
        assert_eq!(value["change"]["schema"], "spec-driven");
        assert!(value.get("initiative").is_none());
    }

    #[test]
    fn test_run_new_change_writes_goal_and_affected_areas() {
        let temp = tempfile::tempdir().unwrap();

        let result = crate::test_support::with_current_dir(temp.path(), || {
            run_new_change(NewChangeOptions {
                name: "add-telemetry",
                description: Some("Add telemetry"),
                schema: Some("spec-driven"),
                goal: Some("Ship telemetry"),
                affected_areas: Some("cli, docs"),
                initiative: None,
                store: None,
                store_path: None,
                json: false,
            })
        });

        result.unwrap();

        let metadata_path = temp
            .path()
            .join("openspec")
            .join("changes")
            .join("add-telemetry")
            .join(".openspec.yaml");
        let metadata: ChangeMetadata =
            serde_yaml::from_str(&std::fs::read_to_string(&metadata_path).unwrap()).unwrap();
        assert_eq!(metadata.schema, "spec-driven");
        assert_eq!(metadata.goal.as_deref(), Some("Ship telemetry"));
        assert_eq!(
            metadata.affected_areas.as_deref(),
            Some(&["cli".to_string(), "docs".to_string()][..])
        );
        assert!(metadata.initiative.is_none());

        let readme = std::fs::read_to_string(
            temp.path()
                .join("openspec")
                .join("changes")
                .join("add-telemetry")
                .join("README.md"),
        )
        .unwrap();
        assert!(readme.contains("Add telemetry"));
    }
}
