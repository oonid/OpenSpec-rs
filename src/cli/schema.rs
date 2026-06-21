use std::path::Path;

use crate::cli::args::SchemaCommands;
use crate::core::error::{OpenSpecError, Result};
use crate::core::schema::{
    get_package_schemas_dir, get_project_schemas_dir, get_user_schemas_dir, list_schemas,
    resolve_schema, SchemaSource,
};

const DEFAULT_SCHEMA: &str = "spec-driven";

fn source_label(source: &SchemaSource) -> &'static str {
    match source {
        SchemaSource::Project => "project",
        SchemaSource::User => "user",
        SchemaSource::Package => "package",
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct ShadowJson {
    source: String,
    path: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ResolutionJson {
    name: String,
    source: String,
    path: String,
    shadows: Vec<ShadowJson>,
}

/// Inspect all three schema locations for `name`, returning the active source
/// (first that exists, in project > user > package precedence) followed by any
/// shadowed locations.
fn resolve_with_shadows(name: &str, project_root: &Path) -> Option<ResolutionJson> {
    let candidates = [
        (
            SchemaSource::Project,
            get_project_schemas_dir(project_root).join(name),
        ),
        (SchemaSource::User, get_user_schemas_dir().join(name)),
        (SchemaSource::Package, get_package_schemas_dir().join(name)),
    ];

    let mut existing: Vec<(SchemaSource, String)> = Vec::new();
    for (source, dir) in candidates {
        if dir.join("schema.yaml").exists() {
            existing.push((source, dir.display().to_string()));
        }
    }

    // Embedded fallback for spec-driven when no on-disk schema is present.
    if existing.is_empty() && name == DEFAULT_SCHEMA {
        return Some(ResolutionJson {
            name: name.to_string(),
            source: "package".to_string(),
            path: "embedded:spec-driven.yaml".to_string(),
            shadows: Vec::new(),
        });
    }

    if existing.is_empty() {
        return None;
    }

    let (active_source, active_path) = existing[0].clone();
    let shadows = existing[1..]
        .iter()
        .map(|(s, p)| ShadowJson {
            source: source_label(s).to_string(),
            path: p.clone(),
        })
        .collect();

    Some(ResolutionJson {
        name: name.to_string(),
        source: source_label(&active_source).to_string(),
        path: active_path,
        shadows,
    })
}

pub fn run(cmd: SchemaCommands) -> Result<()> {
    eprintln!("Note: Schema commands are experimental and may change.");
    match cmd {
        SchemaCommands::Which { name, all, json } => run_which(name.as_deref(), all, json),
        SchemaCommands::Validate { name, json } => run_validate(name.as_deref(), json),
    }
}

fn run_which(name: Option<&str>, all: bool, json: bool) -> Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    if all {
        let schemas = list_schemas(Some(&project_root));
        let resolutions: Vec<ResolutionJson> = schemas
            .iter()
            .filter_map(|s| resolve_with_shadows(&s.name, &project_root))
            .collect();

        if json {
            println!("{}", serde_json::to_string_pretty(&resolutions)?);
        } else if resolutions.is_empty() {
            println!("No schemas found.");
        } else {
            print_grouped(&resolutions, "project", "Project schemas:");
            print_grouped(&resolutions, "user", "User schemas:");
            print_grouped(&resolutions, "package", "Package schemas:");
        }
        return Ok(());
    }

    let name = match name {
        Some(n) => n,
        None => {
            return Err(OpenSpecError::Custom(
                "Schema name is required (or use --all to list all schemas)".to_string(),
            ));
        }
    };

    let resolution = match resolve_with_shadows(name, &project_root) {
        Some(r) => r,
        None => {
            let available = list_schemas(Some(&project_root))
                .into_iter()
                .map(|s| s.name)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(OpenSpecError::schema_not_found(name, available));
        }
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&resolution)?);
    } else {
        println!("Schema: {}", resolution.name);
        println!("Source: {}", resolution.source);
        println!("Path: {}", resolution.path);
        if !resolution.shadows.is_empty() {
            println!();
            println!("Shadows:");
            for shadow in &resolution.shadows {
                println!("  {}: {}", shadow.source, shadow.path);
            }
        }
    }

    Ok(())
}

fn print_grouped(resolutions: &[ResolutionJson], source: &str, heading: &str) {
    let group: Vec<&ResolutionJson> = resolutions.iter().filter(|r| r.source == source).collect();
    if group.is_empty() {
        return;
    }
    println!();
    println!("{}", heading);
    for schema in group {
        if schema.shadows.is_empty() {
            println!("  {}", schema.name);
        } else {
            let sources: Vec<&str> = schema.shadows.iter().map(|s| s.source.as_str()).collect();
            println!("  {} (shadows: {})", schema.name, sources.join(", "));
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct ValidateJson {
    name: String,
    valid: bool,
    issues: Vec<String>,
}

fn run_validate(name: Option<&str>, json: bool) -> Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    let schema_name = name.unwrap_or(DEFAULT_SCHEMA);

    // Resolution surfaces structural/cycle errors as a load failure; treat those
    // as validation issues rather than propagating, so output stays consistent.
    let resolved = match resolve_schema(schema_name, Some(&project_root)) {
        Ok(r) => r,
        Err(OpenSpecError::SchemaNotFound { .. }) => {
            let available = list_schemas(Some(&project_root))
                .into_iter()
                .map(|s| s.name)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(OpenSpecError::schema_not_found(schema_name, available));
        }
        Err(e) => {
            let issues = vec![e.to_string()];
            return emit_validate(schema_name, false, issues, json);
        }
    };

    match resolved.schema.validate() {
        Ok(()) => emit_validate(schema_name, true, Vec::new(), json),
        Err(message) => emit_validate(schema_name, false, vec![message], json),
    }
}

fn emit_validate(name: &str, valid: bool, issues: Vec<String>, json: bool) -> Result<()> {
    if json {
        let output = ValidateJson {
            name: name.to_string(),
            valid,
            issues: issues.clone(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else if valid {
        println!("Schema '{}' is valid", name);
    } else {
        println!("Schema '{}' has errors:", name);
        for issue in &issues {
            println!("  error: {}", issue);
        }
    }

    if valid {
        Ok(())
    } else {
        Err(OpenSpecError::Custom(format!(
            "Schema '{}' is invalid",
            name
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_driven_is_valid() {
        let resolved = resolve_schema("spec-driven", None).unwrap();
        assert!(resolved.schema.validate().is_ok());
    }

    #[test]
    fn test_resolve_with_shadows_embedded() {
        // With no project root, spec-driven falls back to the embedded schema
        // when no on-disk package copy is present.
        let dir = std::path::PathBuf::from("/nonexistent-project-root-xyz");
        let resolution = resolve_with_shadows("spec-driven", &dir);
        assert!(resolution.is_some());
        let resolution = resolution.unwrap();
        assert_eq!(resolution.name, "spec-driven");
        assert_eq!(resolution.source, "package");
    }

    #[test]
    fn test_resolve_with_shadows_missing() {
        let dir = std::path::PathBuf::from("/nonexistent-project-root-xyz");
        assert!(resolve_with_shadows("does-not-exist", &dir).is_none());
    }
}
