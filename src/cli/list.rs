use std::path::PathBuf;

use crate::core::config::OPENSPEC_DIR_NAME;
use crate::core::error::{OpenSpecError, Result};
use crate::core::spec_parser::SpecParser;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChangeInfo {
    pub name: String,
    pub completed_tasks: usize,
    pub total_tasks: usize,
    pub last_modified: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SpecInfo {
    pub id: String,
    pub requirement_count: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ListChangesOutput {
    pub changes: Vec<ChangeInfoJson>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChangeInfoJson {
    pub name: String,
    pub completed_tasks: usize,
    pub total_tasks: usize,
    pub last_modified: String,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ListSpecsOutput {
    pub specs: Vec<SpecInfoJson>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SpecInfoJson {
    pub id: String,
    pub requirement_count: usize,
}

pub fn run_list(specs: bool, _changes: bool, sort: &str, json: bool) -> Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    if specs {
        list_specs(&project_root, json)
    } else {
        list_changes(&project_root, sort, json)
    }
}

fn list_changes(project_root: &std::path::Path, sort: &str, json: bool) -> Result<()> {
    let changes_dir = project_root.join(OPENSPEC_DIR_NAME).join("changes");

    if !changes_dir.exists() {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&ListChangesOutput { changes: vec![] })?
            );
        } else {
            println!("No active changes found.");
        }
        return Ok(());
    }

    let entries = std::fs::read_dir(&changes_dir)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to read changes directory: {}", e)))?;

    let mut change_dirs: Vec<String> = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|e| OpenSpecError::Custom(format!("Failed to read entry: {}", e)))?;
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name != "archive" && !name.starts_with('.') {
                change_dirs.push(name);
            }
        }
    }

    if change_dirs.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&ListChangesOutput { changes: vec![] })?
            );
        } else {
            println!("No active changes found.");
        }
        return Ok(());
    }

    let mut changes: Vec<ChangeInfo> = Vec::new();
    for change_name in &change_dirs {
        let change_path = changes_dir.join(change_name);
        let (completed, total) = count_tasks(&change_path)?;
        let last_modified = get_last_modified(&change_path)?;

        changes.push(ChangeInfo {
            name: change_name.clone(),
            completed_tasks: completed,
            total_tasks: total,
            last_modified,
        });
    }

    if sort == "recent" {
        changes.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    } else {
        changes.sort_by(|a, b| a.name.cmp(&b.name));
    }

    if json {
        let json_changes: Vec<ChangeInfoJson> = changes
            .iter()
            .map(|c| {
                let status = if c.total_tasks == 0 {
                    "no-tasks".to_string()
                } else if c.completed_tasks == c.total_tasks {
                    "complete".to_string()
                } else {
                    "in-progress".to_string()
                };
                ChangeInfoJson {
                    name: c.name.clone(),
                    completed_tasks: c.completed_tasks,
                    total_tasks: c.total_tasks,
                    last_modified: c.last_modified.clone(),
                    status,
                }
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&ListChangesOutput {
                changes: json_changes,
            })?
        );
    } else {
        println!("Changes:");
        let name_width = changes.iter().map(|c| c.name.len()).max().unwrap_or(0);
        for change in &changes {
            let padded_name = pad_right(&change.name, name_width);
            let status = format_task_status(change.completed_tasks, change.total_tasks);
            let time_ago = format_relative_time(&change.last_modified);
            println!("  {}     {:12}  {}", padded_name, status, time_ago);
        }
    }

    Ok(())
}

fn list_specs(project_root: &std::path::Path, json: bool) -> Result<()> {
    let specs_dir = project_root.join(OPENSPEC_DIR_NAME).join("specs");

    if !specs_dir.exists() {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&ListSpecsOutput { specs: vec![] })?
            );
        } else {
            println!("No specs found.");
        }
        return Ok(());
    }

    let entries = std::fs::read_dir(&specs_dir)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to read specs directory: {}", e)))?;

    let mut spec_dirs: Vec<String> = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|e| OpenSpecError::Custom(format!("Failed to read entry: {}", e)))?;
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            spec_dirs.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    if spec_dirs.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&ListSpecsOutput { specs: vec![] })?
            );
        } else {
            println!("No specs found.");
        }
        return Ok(());
    }

    let mut specs: Vec<SpecInfo> = Vec::new();
    for spec_id in &spec_dirs {
        let spec_path = specs_dir.join(spec_id).join("spec.md");
        let requirement_count = if spec_path.exists() {
            match std::fs::read_to_string(&spec_path) {
                Ok(content) => {
                    let mut parser = SpecParser::new(&content);
                    match parser.parse_spec(spec_id) {
                        Ok(spec) => spec.requirements.len(),
                        Err(_) => 0,
                    }
                }
                Err(_) => 0,
            }
        } else {
            0
        };
        specs.push(SpecInfo {
            id: spec_id.clone(),
            requirement_count,
        });
    }

    specs.sort_by(|a, b| a.id.cmp(&b.id));

    if json {
        let json_specs: Vec<SpecInfoJson> = specs
            .iter()
            .map(|s| SpecInfoJson {
                id: s.id.clone(),
                requirement_count: s.requirement_count,
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&ListSpecsOutput { specs: json_specs })?
        );
    } else {
        println!("Specs:");
        let id_width = specs.iter().map(|s| s.id.len()).max().unwrap_or(0);
        for spec in &specs {
            let padded = pad_right(&spec.id, id_width);
            println!("  {}     requirements {}", padded, spec.requirement_count);
        }
    }

    Ok(())
}

fn pad_right(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{:width$}", s, width = width)
    }
}

fn count_tasks(change_dir: &std::path::Path) -> Result<(usize, usize)> {
    let tasks_path = change_dir.join("tasks.md");
    if !tasks_path.exists() {
        return Ok((0, 0));
    }

    let content = std::fs::read_to_string(&tasks_path)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to read tasks file: {}", e)))?;

    let mut total = 0;
    let mut completed = 0;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("- [ ] ") || line.starts_with("* [ ] ") {
            total += 1;
        } else if line.starts_with("- [x] ")
            || line.starts_with("- [X] ")
            || line.starts_with("* [x] ")
            || line.starts_with("* [X] ")
        {
            total += 1;
            completed += 1;
        }
    }

    Ok((completed, total))
}

fn get_last_modified(dir: &PathBuf) -> Result<String> {
    let mut latest: Option<std::time::SystemTime> = None;

    fn walk(current_dir: &PathBuf, latest_ref: &mut Option<std::time::SystemTime>) -> Result<()> {
        let entries = std::fs::read_dir(current_dir)
            .map_err(|e| OpenSpecError::Custom(format!("Failed to read directory: {}", e)))?;

        for entry in entries {
            let entry =
                entry.map_err(|e| OpenSpecError::Custom(format!("Failed to read entry: {}", e)))?;
            let path = current_dir.join(entry.file_name());

            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                walk(&path, latest_ref)?;
            } else {
                let metadata = std::fs::metadata(&path).map_err(|e| {
                    OpenSpecError::Custom(format!("Failed to read metadata: {}", e))
                })?;
                if let Ok(m) = metadata.modified() {
                    if latest_ref.is_none() || m > latest_ref.unwrap() {
                        *latest_ref = Some(m);
                    }
                }
            }
        }
        Ok(())
    }

    walk(dir, &mut latest)?;

    let last_modified = match latest {
        Some(time) => {
            let datetime: chrono::DateTime<chrono::Utc> = time.into();
            datetime.to_rfc3339()
        }
        None => {
            let metadata = std::fs::metadata(dir).map_err(|e| {
                OpenSpecError::Custom(format!("Failed to read directory metadata: {}", e))
            })?;
            let datetime: chrono::DateTime<chrono::Utc> = metadata.modified().ok().unwrap().into();
            datetime.to_rfc3339()
        }
    };

    Ok(last_modified)
}

fn format_task_status(completed: usize, total: usize) -> String {
    if total == 0 {
        "no-tasks".to_string()
    } else if completed == total {
        "complete".to_string()
    } else {
        format!("{}/{}", completed, total)
    }
}

fn format_relative_time(iso_time: &str) -> String {
    let datetime = chrono::DateTime::parse_from_rfc3339(iso_time);
    if datetime.is_err() {
        return iso_time.to_string();
    }
    let datetime = datetime.unwrap().with_timezone(&chrono::Utc);
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(datetime);

    let days = duration.num_days();
    let hours = duration.num_hours();
    let mins = duration.num_minutes();

    if days > 30 {
        datetime.format("%Y-%m-%d").to_string()
    } else if days > 0 {
        format!("{}d ago", days)
    } else if hours > 0 {
        format!("{}h ago", hours)
    } else if mins > 0 {
        format!("{}m ago", mins)
    } else {
        "just now".to_string()
    }
}
