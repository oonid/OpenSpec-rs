use crate::core::config::OPENSPEC_DIR_NAME;
use crate::core::error::{OpenSpecError, Result};

#[derive(Debug, Clone, serde::Serialize)]
pub struct RequirementJson {
    pub text: String,
    pub scenarios: Vec<ScenarioJson>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScenarioJson {
    #[serde(rename = "rawText")]
    pub raw_text: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RenameJson {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Delta {
    pub spec: String,
    pub operation: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirement: Option<RequirementJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename: Option<RenameJson>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ShowChangeOutput {
    pub id: String,
    pub title: String,
    #[serde(rename = "deltaCount")]
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
        let deltas = build_deltas_from_spec_files(&change_dir);

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
        struct SpecMetadataOutput {
            version: String,
            format: String,
            #[serde(rename = "sourcePath", skip_serializing_if = "Option::is_none")]
            source_path: Option<String>,
        }

        #[derive(Debug, Clone, serde::Serialize)]
        struct SpecOutput {
            id: String,
            title: String,
            overview: String,
            #[serde(rename = "requirementCount")]
            requirement_count: usize,
            requirements: Vec<RequirementJson>,
            metadata: SpecMetadataOutput,
        }

        let metadata = match &spec.metadata {
            Some(m) => SpecMetadataOutput {
                version: m.version.clone(),
                format: m.format.clone(),
                source_path: m.source_path.clone(),
            },
            None => SpecMetadataOutput {
                version: "1.0.0".to_string(),
                format: "openspec".to_string(),
                source_path: None,
            },
        };

        let output = SpecOutput {
            id: spec_id.to_string(),
            title: spec.name,
            overview: spec.overview,
            requirement_count: spec.requirements.len(),
            requirements: spec
                .requirements
                .iter()
                .map(|r| RequirementJson {
                    text: r.text.clone(),
                    scenarios: r
                        .scenarios
                        .iter()
                        .map(|s| ScenarioJson {
                            raw_text: s.raw_text.clone(),
                        })
                        .collect(),
                })
                .collect(),
            metadata,
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

/// Build the change's deltas by scanning the delta spec files under
/// `<change_dir>/specs/<capability>/spec.md`, mirroring upstream
/// ChangeParser.parseDeltaSpecs / parseSpecDeltas. For each spec dir we emit one
/// delta per ADDED/MODIFIED/REMOVED requirement and one per RENAMED pair, in that
/// order, iterating spec dirs in sorted order.
fn build_deltas_from_spec_files(change_dir: &std::path::Path) -> Vec<Delta> {
    use crate::core::spec_parser::{parse_delta_spec, SpecParser};

    let mut deltas = Vec::new();
    let specs_dir = change_dir.join("specs");

    let mut spec_dirs: Vec<(String, std::path::PathBuf)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&specs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let spec_file = path.join("spec.md");
                if spec_file.exists() {
                    let name = path.file_name().map(|n| n.to_string_lossy().to_string());
                    if let Some(name) = name {
                        spec_dirs.push((name, spec_file));
                    }
                }
            }
        }
    }
    spec_dirs.sort_by(|a, b| a.0.cmp(&b.0));

    let req_json = |r: &crate::core::spec_parser::Requirement| RequirementJson {
        text: r.text.clone(),
        scenarios: r
            .scenarios
            .iter()
            .map(|s| ScenarioJson {
                raw_text: s.raw_text.clone(),
            })
            .collect(),
    };

    for (spec_name, spec_file) in &spec_dirs {
        let content = match std::fs::read_to_string(spec_file) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let parser = SpecParser::new(&content);

        for req in parser.parse_delta_section_requirements("ADDED Requirements") {
            deltas.push(Delta {
                spec: spec_name.clone(),
                operation: "ADDED".to_string(),
                description: format!("Add requirement: {}", req.text),
                requirement: Some(req_json(&req)),
                rename: None,
            });
        }

        for req in parser.parse_delta_section_requirements("MODIFIED Requirements") {
            deltas.push(Delta {
                spec: spec_name.clone(),
                operation: "MODIFIED".to_string(),
                description: format!("Modify requirement: {}", req.text),
                requirement: Some(req_json(&req)),
                rename: None,
            });
        }

        for req in parser.parse_delta_section_requirements("REMOVED Requirements") {
            deltas.push(Delta {
                spec: spec_name.clone(),
                operation: "REMOVED".to_string(),
                description: format!("Remove requirement: {}", req.text),
                requirement: Some(req_json(&req)),
                rename: None,
            });
        }

        let plan = parse_delta_spec(&content);
        for pair in &plan.renamed {
            deltas.push(Delta {
                spec: spec_name.clone(),
                operation: "RENAMED".to_string(),
                description: format!(
                    "Rename requirement from \"{}\" to \"{}\"",
                    pair.from, pair.to
                ),
                requirement: None,
                rename: Some(RenameJson {
                    from: pair.from.clone(),
                    to: pair.to.clone(),
                }),
            });
        }
    }

    deltas
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renamed_delta_serializes_with_rename_object() {
        let delta = Delta {
            spec: "cli-commands".to_string(),
            operation: "RENAMED".to_string(),
            description: "Rename requirement from \"Old\" to \"New\"".to_string(),
            requirement: None,
            rename: Some(RenameJson {
                from: "Old".to_string(),
                to: "New".to_string(),
            }),
        };

        let v: serde_json::Value = serde_json::to_value(&delta).unwrap();
        assert_eq!(v["operation"], "RENAMED");
        assert_eq!(v["rename"]["from"], "Old");
        assert_eq!(v["rename"]["to"], "New");
        // No "RENAMED from X" string and no requirement object.
        assert!(v.get("requirement").is_none());
        assert!(v["operation"].as_str().unwrap() == "RENAMED");
    }

    #[test]
    fn test_added_delta_includes_requirement_with_raw_text_scenarios() {
        let delta = Delta {
            spec: "telemetry".to_string(),
            operation: "ADDED".to_string(),
            description: "Add requirement: The system SHALL collect telemetry.".to_string(),
            requirement: Some(RequirementJson {
                text: "The system SHALL collect telemetry.".to_string(),
                scenarios: vec![ScenarioJson {
                    raw_text: "- **WHEN** enabled\n- **THEN** events are sent".to_string(),
                }],
            }),
            rename: None,
        };

        let v: serde_json::Value = serde_json::to_value(&delta).unwrap();
        assert_eq!(v["operation"], "ADDED");
        assert_eq!(
            v["requirement"]["text"],
            "The system SHALL collect telemetry."
        );
        assert_eq!(
            v["requirement"]["scenarios"][0]["rawText"],
            "- **WHEN** enabled\n- **THEN** events are sent"
        );
        // camelCase key, not snake_case.
        assert!(v["requirement"]["scenarios"][0].get("raw_text").is_none());
        assert!(v.get("rename").is_none());
    }

    #[test]
    fn test_build_deltas_from_spec_files_counts_and_shape() {
        let temp = tempfile::tempdir().unwrap();
        let change_dir = temp.path().join("changes").join("my-change");
        let added_dir = change_dir.join("specs").join("telemetry");
        let modified_dir = change_dir.join("specs").join("cli-commands");
        std::fs::create_dir_all(&added_dir).unwrap();
        std::fs::create_dir_all(&modified_dir).unwrap();

        std::fs::write(
            added_dir.join("spec.md"),
            "## ADDED Requirements\n\n### Requirement: Collect Telemetry\nThe system SHALL collect telemetry.\n\n#### Scenario: Enabled\n- **WHEN** enabled\n- **THEN** events are sent\n",
        )
        .unwrap();

        std::fs::write(
            modified_dir.join("spec.md"),
            "## MODIFIED Requirements\n\n### Requirement: Show Command\nThe show command SHALL emit JSON.\n\n#### Scenario: Json\n- **WHEN** --json is passed\n- **THEN** JSON is printed\n",
        )
        .unwrap();

        let deltas = build_deltas_from_spec_files(&change_dir);
        assert_eq!(deltas.len(), 2);

        // Sorted by spec dir name: cli-commands before telemetry.
        assert_eq!(deltas[0].spec, "cli-commands");
        assert_eq!(deltas[0].operation, "MODIFIED");
        assert!(deltas[0].requirement.is_some());
        assert_eq!(
            deltas[0].requirement.as_ref().unwrap().text,
            "The show command SHALL emit JSON."
        );
        assert_eq!(deltas[0].requirement.as_ref().unwrap().scenarios.len(), 1);

        assert_eq!(deltas[1].spec, "telemetry");
        assert_eq!(deltas[1].operation, "ADDED");
        assert_eq!(
            deltas[1].description,
            "Add requirement: The system SHALL collect telemetry."
        );

        let output = ShowChangeOutput {
            id: "my-change".to_string(),
            title: "My Change".to_string(),
            delta_count: deltas.len(),
            deltas,
        };
        let v: serde_json::Value = serde_json::to_value(&output).unwrap();
        assert_eq!(v["deltaCount"], 2);
        assert!(v.get("delta_count").is_none());
    }
}
