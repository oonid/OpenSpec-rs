use regex::Regex;

use crate::core::error::{OpenSpecError, Result};

pub const DEFAULT_SCHEMA: &str = "spec-driven";

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

pub fn run_new_change(name: &str, description: Option<&str>, schema: Option<&str>) -> Result<()> {
    let validation = validate_change_name(name);
    if !validation.valid {
        return Err(OpenSpecError::Custom(
            validation
                .error
                .unwrap_or_else(|| "Invalid change name".to_string()),
        ));
    }

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

    let today = chrono_lite_today();
    let metadata_content = format!("schema: {}\ncreated: {}\n", schema_name, today);
    let metadata_path = change_dir.join(".openspec.yaml");
    std::fs::write(&metadata_path, metadata_content)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to write metadata file: {}", e)))?;

    if let Some(desc) = description {
        let readme_content = format!("# {}\n\n{}\n", name, desc);
        let readme_path = change_dir.join("README.md");
        std::fs::write(&readme_path, readme_content)
            .map_err(|e| OpenSpecError::Custom(format!("Failed to write README file: {}", e)))?;
    }

    println!(
        "Created change '{}' at openspec/changes/{}/ (schema: {})",
        name, name, schema_name
    );

    Ok(())
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
