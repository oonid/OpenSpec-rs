use std::collections::HashSet;
use std::path::PathBuf;

use crate::core::artifact::{
    ArtifactGraph, ArtifactStatus, ArtifactStatusKind, ChangeStatus, CompletedSet,
};
use crate::core::config::OPENSPEC_DIR_NAME;
use crate::core::error::{OpenSpecError, Result};
use crate::core::schema::{load_schema, parse_schema, SchemaYaml};

pub fn run_status(change: Option<&str>, schema_override: Option<&str>, json: bool) -> Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    if change.is_none() {
        let available = get_available_changes(&project_root)?;
        if available.is_empty() {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "changes": [],
                        "message": "No active changes."
                    }))
                    .unwrap()
                );
            } else {
                println!("No active changes. Create one with: openspec new change <name>");
            }
            return Ok(());
        }
        return Err(OpenSpecError::Custom(format!(
            "Missing required option --change. Available changes:\n  {}",
            available.join("\n  ")
        )));
    }

    let change_name = change.unwrap();
    let change_dir = find_change_dir(&project_root, change_name)?;

    let schema_name = get_change_schema(&change_dir, schema_override)?;
    let schema = load_schema_for_change(&project_root, &schema_name)?;

    let completed = detect_completed_artifacts(&change_dir, &schema)?;
    let status = compute_change_status(&schema, completed, change_name);

    if json {
        println!("{}", status.format_json().unwrap());
    } else {
        let mut stdout = std::io::stdout();
        status
            .format_text(&mut stdout)
            .map_err(|e| OpenSpecError::Custom(format!("Failed to write output: {}", e)))?;
    }

    Ok(())
}

fn compute_change_status(
    schema: &SchemaYaml,
    completed: CompletedSet,
    change_name: &str,
) -> ChangeStatus {
    let graph = ArtifactGraph::new(schema);

    let apply_requires: Vec<String> = schema
        .apply
        .as_ref()
        .map(|a| a.requires.clone())
        .unwrap_or_else(|| schema.artifacts.iter().map(|a| a.id.clone()).collect());

    let ready: HashSet<String> = graph.get_next_artifacts(&completed).into_iter().collect();
    let blocked = graph.get_blocked(&completed);

    let artifact_statuses: Vec<ArtifactStatus> = graph
        .get_all_artifacts()
        .iter()
        .map(|artifact| {
            if completed.contains(&artifact.id) {
                ArtifactStatus {
                    id: artifact.id.clone(),
                    output_path: artifact.generates.clone(),
                    status: ArtifactStatusKind::Done,
                    missing_deps: vec![],
                }
            } else if ready.contains(&artifact.id) {
                ArtifactStatus {
                    id: artifact.id.clone(),
                    output_path: artifact.generates.clone(),
                    status: ArtifactStatusKind::Ready,
                    missing_deps: vec![],
                }
            } else {
                let missing = blocked.get(&artifact.id).cloned().unwrap_or_default();
                ArtifactStatus {
                    id: artifact.id.clone(),
                    output_path: artifact.generates.clone(),
                    status: ArtifactStatusKind::Blocked,
                    missing_deps: missing,
                }
            }
        })
        .collect();

    let is_complete = graph.get_all_artifacts().len() == completed.len();

    ChangeStatus {
        change_name: change_name.to_string(),
        schema_name: schema.name.clone(),
        is_complete,
        apply_requires,
        artifacts: artifact_statuses,
    }
}

fn get_available_changes(project_root: &std::path::Path) -> Result<Vec<String>> {
    let changes_dir = project_root.join(OPENSPEC_DIR_NAME).join("changes");

    if !changes_dir.exists() {
        return Ok(vec![]);
    }

    let mut changes = Vec::new();

    let entries = std::fs::read_dir(&changes_dir)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to read changes directory: {}", e)))?;

    for entry in entries {
        let entry =
            entry.map_err(|e| OpenSpecError::Custom(format!("Failed to read entry: {}", e)))?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name != "archive" && path.join(".openspec.yaml").exists() {
                    changes.push(name.to_string());
                }
            }
        }
    }

    changes.sort();
    Ok(changes)
}

fn find_change_dir(project_root: &std::path::Path, change_name: &str) -> Result<PathBuf> {
    let change_dir = project_root
        .join(OPENSPEC_DIR_NAME)
        .join("changes")
        .join(change_name);

    if !change_dir.exists() {
        return Err(OpenSpecError::Custom(format!(
            "Change '{}' not found",
            change_name
        )));
    }

    Ok(change_dir)
}

fn get_change_schema(
    change_dir: &std::path::Path,
    schema_override: Option<&str>,
) -> Result<String> {
    if let Some(schema) = schema_override {
        return Ok(schema.to_string());
    }

    let metadata_path = change_dir.join(".openspec.yaml");
    if metadata_path.exists() {
        let content = std::fs::read_to_string(&metadata_path)
            .map_err(|e| OpenSpecError::Custom(format!("Failed to read .openspec.yaml: {}", e)))?;

        for line in content.lines() {
            let line = line.trim();
            if let Some(stripped) = line.strip_prefix("schema:") {
                let schema: String = stripped.trim().to_string();
                return Ok(schema);
            }
        }
    }

    Ok("spec-driven".to_string())
}

fn load_schema_for_change(project_root: &std::path::Path, schema_name: &str) -> Result<SchemaYaml> {
    let project_schema_path = project_root
        .join(OPENSPEC_DIR_NAME)
        .join("schemas")
        .join(format!("{}.yaml", schema_name));

    if project_schema_path.exists() {
        return load_schema(&project_schema_path);
    }

    if schema_name == "spec-driven" {
        return load_embedded_spec_driven_schema();
    }

    Err(OpenSpecError::Custom(format!(
        "Schema '{}' not found",
        schema_name
    )))
}

fn load_embedded_spec_driven_schema() -> Result<SchemaYaml> {
    let schema_content = include_str!("../embedded_schemas/spec-driven.yaml");
    parse_schema(schema_content, "embedded spec-driven schema".to_string())
}

fn detect_completed_artifacts(
    change_dir: &std::path::Path,
    schema: &SchemaYaml,
) -> Result<CompletedSet> {
    let mut completed = HashSet::new();

    for artifact in &schema.artifacts {
        let output_path = change_dir.join(&artifact.generates);

        if artifact.generates.contains('*') {
            if has_glob_matches(change_dir, &artifact.generates)? {
                completed.insert(artifact.id.clone());
            }
        } else if output_path.exists() {
            completed.insert(artifact.id.clone());
        }
    }

    Ok(completed)
}

fn has_glob_matches(base_dir: &std::path::Path, pattern: &str) -> Result<bool> {
    let full_pattern = base_dir.join(pattern);
    let pattern_str = full_pattern.to_string_lossy();

    match glob::glob(&pattern_str) {
        Ok(paths) => {
            for path in paths {
                if path.is_ok() {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Err(_) => Ok(false),
    }
}
