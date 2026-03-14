use crate::core::config::OPENSPEC_DIR_NAME;
use crate::core::error::{OpenSpecError, Result};
use crate::core::spec_parser::parse_delta_spec;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Delta {
    pub operation: String,
    pub requirement: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ShowChangeOutput {
    pub id: String,
    pub title: String,
    pub delta_count: usize,
    pub deltas: Vec<Delta>,
}

pub fn run_show(name: &str, item_type: Option<&str>, json: bool, _deltas_only: bool) -> Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    let changes = get_available_changes(&project_root)?;
    let specs = get_available_specs(&project_root)?;

    let is_change = changes.contains(&name.to_string());
    let is_spec = specs.contains(&name.to_string());

    let resolved_type = item_type.unwrap_or(if is_change {
        "change"
    } else if is_spec {
        "spec"
    } else {
        "unknown"
    });

    if resolved_type == "unknown" {
        return Err(OpenSpecError::Custom(format!(
            "Unknown item '{}'. Available changes: {}, Available specs: {}",
            name,
            changes.join(", "),
            specs.join(", ")
        )));
    }

    if item_type.is_none() && is_change && is_spec {
        return Err(OpenSpecError::Custom(format!(
            "Ambiguous item '{}' matches both a change and a spec. Use --type change|spec",
            name
        )));
    }

    match resolved_type {
        "change" => show_change(&project_root, name, json),
        "spec" => show_spec(&project_root, name, json),
        _ => Err(OpenSpecError::Custom(format!(
            "Unknown type '{}'. Use 'change' or 'spec'",
            resolved_type
        ))),
    }
}

fn show_change(project_root: &std::path::Path, change_name: &str, json: bool) -> Result<()> {
    let change_dir = project_root
        .join(OPENSPEC_DIR_NAME)
        .join("changes")
        .join(change_name);

    let proposal_path = change_dir.join("proposal.md");
    let readme_path = change_dir.join("README.md");

    let (_content_path, content) = if proposal_path.exists() {
        let c = std::fs::read_to_string(&proposal_path)
            .map_err(|e| OpenSpecError::Custom(format!("Failed to read proposal: {}", e)))?;
        (proposal_path, c)
    } else if readme_path.exists() {
        let c = std::fs::read_to_string(&readme_path)
            .map_err(|e| OpenSpecError::Custom(format!("Failed to read README: {}", e)))?;
        (readme_path, c)
    } else {
        return Err(OpenSpecError::Custom(format!(
            "Change '{}' not found - no proposal.md or README.md at {}",
            change_name,
            change_dir.display()
        )));
    };

    if json {
        let title = extract_title(&content, change_name);
        let deltas = parse_deltas_from_proposal(&content);

        let output = ShowChangeOutput {
            id: change_name.to_string(),
            title,
            delta_count: deltas.len(),
            deltas,
        };

        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{}", content);
    }

    Ok(())
}

fn show_spec(project_root: &std::path::Path, spec_id: &str, json: bool) -> Result<()> {
    let spec_path = project_root
        .join(OPENSPEC_DIR_NAME)
        .join("specs")
        .join(spec_id)
        .join("spec.md");

    if !spec_path.exists() {
        return Err(OpenSpecError::Custom(format!(
            "Spec '{}' not found at {}",
            spec_id,
            spec_path.display()
        )));
    }

    let content = std::fs::read_to_string(&spec_path)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to read spec: {}", e)))?;

    if json {
        use crate::core::spec_parser::SpecParser;

        let mut parser = SpecParser::new(&content);
        let spec = parser
            .parse_spec(spec_id)
            .map_err(|e| OpenSpecError::Custom(format!("Failed to parse spec: {}", e)))?;

        #[derive(Debug, Clone, serde::Serialize)]
        struct SpecOutput {
            id: String,
            title: String,
            overview: String,
            requirement_count: usize,
            requirements: Vec<RequirementOutput>,
        }

        #[derive(Debug, Clone, serde::Serialize)]
        struct RequirementOutput {
            text: String,
            scenarios: Vec<ScenarioOutput>,
        }

        #[derive(Debug, Clone, serde::Serialize)]
        struct ScenarioOutput {
            raw_text: String,
        }

        let output = SpecOutput {
            id: spec_id.to_string(),
            title: spec.name,
            overview: spec.overview,
            requirement_count: spec.requirements.len(),
            requirements: spec
                .requirements
                .iter()
                .map(|r| RequirementOutput {
                    text: r.text.clone(),
                    scenarios: r
                        .scenarios
                        .iter()
                        .map(|s| ScenarioOutput {
                            raw_text: s.raw_text.clone(),
                        })
                        .collect(),
                })
                .collect(),
        };

        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{}", content);
    }

    Ok(())
}

fn get_available_changes(project_root: &std::path::Path) -> Result<Vec<String>> {
    let changes_dir = project_root.join(OPENSPEC_DIR_NAME).join("changes");

    if !changes_dir.exists() {
        return Ok(vec![]);
    }

    let mut changes = Vec::new();
    for entry in std::fs::read_dir(&changes_dir)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to read changes directory: {}", e)))?
    {
        let entry =
            entry.map_err(|e| OpenSpecError::Custom(format!("Failed to read entry: {}", e)))?;
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name != "archive" && !name.starts_with('.') {
                let metadata_path = changes_dir.join(&name).join(".openspec.yaml");
                if metadata_path.exists() {
                    changes.push(name);
                }
            }
        }
    }

    changes.sort();
    Ok(changes)
}

fn get_available_specs(project_root: &std::path::Path) -> Result<Vec<String>> {
    let specs_dir = project_root.join(OPENSPEC_DIR_NAME).join("specs");

    if !specs_dir.exists() {
        return Ok(vec![]);
    }

    let mut specs = Vec::new();
    for entry in std::fs::read_dir(&specs_dir)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to read specs directory: {}", e)))?
    {
        let entry =
            entry.map_err(|e| OpenSpecError::Custom(format!("Failed to read entry: {}", e)))?;
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            let name = entry.file_name().to_string_lossy().to_string();
            let spec_path = specs_dir.join(&name).join("spec.md");
            if spec_path.exists() {
                specs.push(name);
            }
        }
    }

    specs.sort();
    Ok(specs)
}

fn extract_title(content: &str, default: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ") {
            let title = title.trim();
            if let Some(rest) = title.strip_prefix("Change:") {
                return rest.trim().to_string();
            }
            if let Some(rest) = title.strip_prefix("change:") {
                return rest.trim().to_string();
            }
            return title.to_string();
        }
    }
    default.to_string()
}

fn parse_deltas_from_proposal(content: &str) -> Vec<Delta> {
    let mut deltas = Vec::new();

    let delta_plan = parse_delta_spec(content);

    for req in &delta_plan.added {
        deltas.push(Delta {
            operation: "ADDED".to_string(),
            requirement: req.name.clone(),
        });
    }

    for req in &delta_plan.modified {
        deltas.push(Delta {
            operation: "MODIFIED".to_string(),
            requirement: req.name.clone(),
        });
    }

    for name in &delta_plan.removed {
        deltas.push(Delta {
            operation: "REMOVED".to_string(),
            requirement: name.clone(),
        });
    }

    for pair in &delta_plan.renamed {
        deltas.push(Delta {
            operation: format!("RENAMED from {}", pair.from),
            requirement: pair.to.clone(),
        });
    }

    deltas
}
