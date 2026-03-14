use std::collections::HashSet;
use std::path::PathBuf;

use crate::core::artifact::{ArtifactGraph, ChangeContext, CompletedSet, DependencyInfo};
use crate::core::config::OPENSPEC_DIR_NAME;
use crate::core::error::{OpenSpecError, Result};
use crate::core::schema::{load_schema, parse_schema, Artifact, SchemaYaml};

#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskItem {
    pub id: String,
    pub description: String,
    pub done: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ApplyInstructions {
    pub change_name: String,
    pub change_dir: String,
    pub schema_name: String,
    pub context_files: std::collections::HashMap<String, String>,
    pub progress: Progress,
    pub tasks: Vec<TaskItem>,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_artifacts: Option<Vec<String>>,
    pub instruction: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Progress {
    pub total: usize,
    pub complete: usize,
    pub remaining: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ArtifactInstructions {
    pub change_name: String,
    pub artifact_id: String,
    pub schema_name: String,
    pub change_dir: String,
    pub output_path: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<String>>,
    pub template: String,
    pub dependencies: Vec<DependencyInfoJson>,
    pub unlocks: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DependencyInfoJson {
    pub id: String,
    pub done: bool,
    pub path: String,
    pub description: String,
}

pub fn run_instructions(
    artifact: Option<&str>,
    change: Option<&str>,
    schema_override: Option<&str>,
    json: bool,
) -> Result<()> {
    let _project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    if artifact.is_none() {
        return Err(OpenSpecError::Custom(
            "Missing required argument <artifact>. Use 'apply' for apply instructions or an artifact ID (proposal, specs, design, tasks).".to_string()
        ));
    }

    let artifact_id = artifact.unwrap();

    if artifact_id == "apply" {
        return run_apply_instructions(change, schema_override, json);
    }

    run_artifact_instructions(artifact_id, change, schema_override, json)
}

pub fn run_artifact_instructions(
    artifact_id: &str,
    change: Option<&str>,
    schema_override: Option<&str>,
    json: bool,
) -> Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    let change_name = change
        .map(|s| s.to_string())
        .or_else(|| {
            let available = get_available_changes(&project_root).ok()?;
            if available.is_empty() {
                None
            } else if available.len() == 1 {
                Some(available[0].clone())
            } else {
                None
            }
        })
        .ok_or_else(|| {
            let available = get_available_changes(&project_root).unwrap_or_default();
            if available.is_empty() {
                OpenSpecError::Custom(
                    "No active changes. Create one with: openspec new change <name>".to_string(),
                )
            } else {
                OpenSpecError::Custom(format!(
                    "Missing required option --change. Available changes:\n  {}",
                    available.join("\n  ")
                ))
            }
        })?;

    let change_dir = find_change_dir(&project_root, &change_name)?;
    let schema_name = get_change_schema(&change_dir, schema_override)?;
    let schema = load_schema_for_change(&project_root, &schema_name)?;
    let completed = detect_completed_artifacts(&change_dir, &schema)?;

    let context = ChangeContext::new(
        &schema,
        completed,
        &change_name,
        change_dir.clone(),
        Some(project_root.clone()),
    );

    let artifact = context
        .graph
        .get_artifact(artifact_id)
        .cloned()
        .ok_or_else(|| {
            let valid_ids: Vec<&str> = context
                .graph
                .get_all_artifacts()
                .iter()
                .map(|a| a.id.as_str())
                .collect();
            OpenSpecError::Custom(format!(
                "Artifact '{}' not found in schema '{}'. Valid artifacts:\n  {}",
                artifact_id,
                context.schema_name,
                valid_ids.join("\n  ")
            ))
        })?;

    let template_content = load_template(&schema_name, &artifact.template, &project_root)?;
    let dependencies = get_dependency_info(&artifact, &context.graph, &context.completed);
    let unlocks = get_unlocked_artifacts(&context.graph, artifact_id);
    let is_blocked = dependencies.iter().any(|d| !d.done);

    let instructions = ArtifactInstructions {
        change_name: change_name.clone(),
        artifact_id: artifact_id.to_string(),
        schema_name: schema_name.clone(),
        change_dir: change_dir.to_string_lossy().to_string(),
        output_path: artifact.generates.clone(),
        description: artifact.description.clone(),
        instruction: artifact.instruction.clone(),
        context: None,
        rules: None,
        template: template_content,
        dependencies: dependencies
            .iter()
            .map(|d| DependencyInfoJson {
                id: d.id.clone(),
                done: d.done,
                path: d.path.clone(),
                description: d.description.clone(),
            })
            .collect(),
        unlocks,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&instructions)?);
    } else {
        print_instructions_text(&instructions, is_blocked)?;
    }

    Ok(())
}

pub fn run_apply_instructions(
    change: Option<&str>,
    schema_override: Option<&str>,
    json: bool,
) -> Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    let change_name = change
        .map(|s| s.to_string())
        .or_else(|| {
            let available = get_available_changes(&project_root).ok()?;
            if available.is_empty() {
                None
            } else if available.len() == 1 {
                Some(available[0].clone())
            } else {
                None
            }
        })
        .ok_or_else(|| {
            let available = get_available_changes(&project_root).unwrap_or_default();
            if available.is_empty() {
                OpenSpecError::Custom(
                    "No active changes. Create one with: openspec new change <name>".to_string(),
                )
            } else {
                OpenSpecError::Custom(format!(
                    "Missing required option --change. Available changes:\n  {}",
                    available.join("\n  ")
                ))
            }
        })?;

    let instructions = generate_apply_instructions(&project_root, &change_name, schema_override)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&instructions)?);
    } else {
        print_apply_instructions_text(&instructions)?;
    }

    Ok(())
}

pub fn generate_apply_instructions(
    project_root: &std::path::Path,
    change_name: &str,
    schema_override: Option<&str>,
) -> Result<ApplyInstructions> {
    let change_dir = find_change_dir(project_root, change_name)?;
    let schema_name = get_change_schema(&change_dir, schema_override)?;
    let schema = load_schema_for_change(project_root, &schema_name)?;

    let apply_config = schema.apply.as_ref();
    let required_artifact_ids: Vec<&str> = apply_config
        .map(|a| a.requires.iter().map(|s| s.as_str()).collect())
        .unwrap_or_else(|| schema.artifacts.iter().map(|a| a.id.as_str()).collect());
    let tracks_file = apply_config.and_then(|a| a.tracks.as_ref());
    let schema_instruction = apply_config.and_then(|a| a.instruction.as_ref());

    let mut missing_artifacts = Vec::new();
    for artifact_id in &required_artifact_ids {
        if let Some(artifact) = schema.artifacts.iter().find(|a| a.id == *artifact_id) {
            if !artifact_output_exists(&change_dir, &artifact.generates)? {
                missing_artifacts.push(artifact_id.to_string());
            }
        }
    }

    let mut context_files = std::collections::HashMap::new();
    for artifact in &schema.artifacts {
        if artifact_output_exists(&change_dir, &artifact.generates)? {
            context_files.insert(
                artifact.id.clone(),
                change_dir
                    .join(&artifact.generates)
                    .to_string_lossy()
                    .to_string(),
            );
        }
    }

    let tasks = if let Some(tracks) = tracks_file {
        let tracks_path = change_dir.join(tracks);
        if tracks_path.exists() {
            let content = std::fs::read_to_string(&tracks_path)
                .map_err(|e| OpenSpecError::Custom(format!("Failed to read tasks file: {}", e)))?;
            parse_tasks_file(&content)
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let total = tasks.len();
    let complete = tasks.iter().filter(|t| t.done).count();
    let remaining = total - complete;

    let (state, instruction) = if !missing_artifacts.is_empty() {
        (
            "blocked".to_string(),
            format!(
                "Cannot apply this change yet. Missing artifacts: {}.\nUse the openspec-continue-change skill to create the missing artifacts first.",
                missing_artifacts.join(", ")
            ),
        )
    } else if tracks_file.is_some() && total == 0 {
        let tracks_filename = tracks_file
            .map(|p| {
                std::path::Path::new(p)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_default();
        (
            "blocked".to_string(),
            format!("The {} file exists but contains no tasks.\nAdd tasks to {} or regenerate it with openspec-continue-change.", tracks_filename, tracks_filename),
        )
    } else if tracks_file.is_some() && remaining == 0 && total > 0 {
        (
            "all_done".to_string(),
            "All tasks are complete! This change is ready to be archived.\nConsider running tests and reviewing the changes before archiving.".to_string(),
        )
    } else if tracks_file.is_none() {
        (
            "ready".to_string(),
            schema_instruction.cloned().unwrap_or_else(|| {
                "All required artifacts complete. Proceed with implementation.".to_string()
            }),
        )
    } else {
        (
            "ready".to_string(),
            schema_instruction.cloned().unwrap_or_else(|| {
                "Read context files, work through pending tasks, mark complete as you go.\nPause if you hit blockers or need clarification.".to_string()
            }),
        )
    };

    Ok(ApplyInstructions {
        change_name: change_name.to_string(),
        change_dir: change_dir.to_string_lossy().to_string(),
        schema_name,
        context_files,
        progress: Progress {
            total,
            complete,
            remaining,
        },
        tasks,
        state,
        missing_artifacts: if missing_artifacts.is_empty() {
            None
        } else {
            Some(missing_artifacts)
        },
        instruction,
    })
}

fn parse_tasks_file(content: &str) -> Vec<TaskItem> {
    let mut tasks = Vec::new();
    let mut task_index = 0;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("- [ ] ") || line.starts_with("- [x] ") || line.starts_with("- [X] ") {
            task_index += 1;
            let done = line.starts_with("- [x] ") || line.starts_with("- [X] ");
            let description = line[6..].trim().to_string();
            tasks.push(TaskItem {
                id: task_index.to_string(),
                description,
                done,
            });
        } else if line.starts_with("* [ ] ")
            || line.starts_with("* [x] ")
            || line.starts_with("* [X] ")
        {
            task_index += 1;
            let done = line.starts_with("* [x] ") || line.starts_with("* [X] ");
            let description = line[6..].trim().to_string();
            tasks.push(TaskItem {
                id: task_index.to_string(),
                description,
                done,
            });
        }
    }

    tasks
}

fn artifact_output_exists(change_dir: &std::path::Path, generates: &str) -> Result<bool> {
    let full_path = change_dir.join(generates);

    if generates.contains('*') {
        let pattern = change_dir.join(generates).to_string_lossy().to_string();
        match glob::glob(&pattern) {
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
    } else {
        Ok(full_path.exists())
    }
}

fn load_template(
    schema_name: &str,
    template_path: &str,
    _project_root: &std::path::Path,
) -> Result<String> {
    if schema_name == "spec-driven" {
        let template_content = match template_path {
            "proposal.md" => include_str!("../embedded_templates/proposal.md"),
            "spec.md" => include_str!("../embedded_templates/spec.md"),
            "design.md" => include_str!("../embedded_templates/design.md"),
            "tasks.md" => include_str!("../embedded_templates/tasks.md"),
            _ => {
                return Err(OpenSpecError::Custom(format!(
                    "Template '{}' not found",
                    template_path
                )))
            }
        };
        Ok(template_content.to_string())
    } else {
        Err(OpenSpecError::Custom(format!(
            "Template loading not implemented for schema '{}'",
            schema_name
        )))
    }
}

fn get_dependency_info(
    artifact: &Artifact,
    graph: &ArtifactGraph,
    completed: &CompletedSet,
) -> Vec<DependencyInfo> {
    artifact
        .requires
        .iter()
        .filter_map(|req_id| {
            graph.get_artifact(req_id).map(|dep| DependencyInfo {
                id: dep.id.clone(),
                done: completed.contains(&dep.id),
                path: dep.generates.clone(),
                description: dep.description.clone(),
            })
        })
        .collect()
}

fn get_unlocked_artifacts(graph: &ArtifactGraph, artifact_id: &str) -> Vec<String> {
    let mut unlocks = Vec::new();

    for artifact in graph.get_all_artifacts() {
        if artifact.requires.contains(&artifact_id.to_string()) {
            unlocks.push(artifact.id.clone());
        }
    }

    unlocks.sort();
    unlocks
}

fn print_instructions_text(instructions: &ArtifactInstructions, is_blocked: bool) -> Result<()> {
    println!(
        "<artifact id=\"{}\" change=\"{}\" schema=\"{}\">",
        instructions.artifact_id, instructions.change_name, instructions.schema_name
    );
    println!();

    if is_blocked {
        let missing: Vec<&str> = instructions
            .dependencies
            .iter()
            .filter(|d| !d.done)
            .map(|d| d.id.as_str())
            .collect();
        println!("<warning>");
        println!(
            "This artifact has unmet dependencies. Complete them first or proceed with caution."
        );
        println!("Missing: {}", missing.join(", "));
        println!("</warning>");
        println!();
    }

    println!("<task>");
    println!(
        "Create the {} artifact for change \"{}\".",
        instructions.artifact_id, instructions.change_name
    );
    println!("{}", instructions.description);
    println!("</task>");
    println!();

    if let Some(ref ctx) = instructions.context {
        println!("<project_context>");
        println!(
            "<!-- This is background information for you. Do NOT include this in your output. -->"
        );
        println!("{}", ctx);
        println!("</project_context>");
        println!();
    }

    if let Some(ref rules) = instructions.rules {
        if !rules.is_empty() {
            println!("<rules>");
            println!("<!-- These are constraints for you to follow. Do NOT include this in your output. -->");
            for rule in rules {
                println!("- {}", rule);
            }
            println!("</rules>");
            println!();
        }
    }

    if !instructions.dependencies.is_empty() {
        println!("<dependencies>");
        println!("Read these files for context before creating this artifact:");
        println!();
        for dep in &instructions.dependencies {
            let status = if dep.done { "done" } else { "missing" };
            let full_path = format!("{}/{}", instructions.change_dir, dep.path);
            println!("<dependency id=\"{}\" status=\"{}\">", dep.id, status);
            println!("  <path>{}</path>", full_path);
            println!("  <description>{}</description>", dep.description);
            println!("</dependency>");
        }
        println!("</dependencies>");
        println!();
    }

    println!("<output>");
    println!(
        "Write to: {}/{}",
        instructions.change_dir, instructions.output_path
    );
    println!("</output>");
    println!();

    if let Some(ref instr) = instructions.instruction {
        println!("<instruction>");
        println!("{}", instr.trim());
        println!("</instruction>");
        println!();
    }

    println!("<template>");
    println!("<!-- Use this as the structure for your output file. Fill in the sections. -->");
    println!("{}", instructions.template.trim());
    println!("</template>");
    println!();

    println!("<success_criteria>");
    println!("<!-- To be defined in schema validation rules -->");
    println!("</success_criteria>");
    println!();

    if !instructions.unlocks.is_empty() {
        println!("<unlocks>");
        println!(
            "Completing this artifact enables: {}",
            instructions.unlocks.join(", ")
        );
        println!("</unlocks>");
        println!();
    }

    println!("</artifact>");

    Ok(())
}

fn print_apply_instructions_text(instructions: &ApplyInstructions) -> Result<()> {
    println!("## Apply: {}", instructions.change_name);
    println!("Schema: {}", instructions.schema_name);
    println!();

    if instructions.state == "blocked" {
        if let Some(ref missing) = instructions.missing_artifacts {
            println!("### ⚠️ Blocked");
            println!();
            println!("Missing artifacts: {}", missing.join(", "));
            println!("Use the openspec-continue-change skill to create these first.");
            println!();
        }
    }

    let context_entries: Vec<_> = instructions.context_files.iter().collect();
    if !context_entries.is_empty() {
        println!("### Context Files");
        for (artifact_id, file_path) in context_entries {
            println!("- {}: {}", artifact_id, file_path);
        }
        println!();
    }

    if instructions.progress.total > 0 || !instructions.tasks.is_empty() {
        println!("### Progress");
        if instructions.state == "all_done" {
            println!(
                "{}/{} complete ✓",
                instructions.progress.complete, instructions.progress.total
            );
        } else {
            println!(
                "{}/{} complete",
                instructions.progress.complete, instructions.progress.total
            );
        }
        println!();
    }

    if !instructions.tasks.is_empty() {
        println!("### Tasks");
        for task in &instructions.tasks {
            let checkbox = if task.done { "[x]" } else { "[ ]" };
            println!("- {} {}", checkbox, task.description);
        }
        println!();
    }

    println!("### Instruction");
    println!("{}", instructions.instruction);

    Ok(())
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
