use crate::core::config::OPENSPEC_DIR_NAME;
use crate::core::error::{OpenSpecError, Result};
use crate::core::spec_parser::{parse_delta_spec, SpecParser};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationIssue {
    pub level: String,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BulkItemResult {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
    #[serde(rename = "durationMs")]
    pub duration_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BulkSummary {
    pub totals: BulkTotals,
    #[serde(rename = "byType")]
    pub by_type: std::collections::HashMap<String, BulkTotals>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BulkTotals {
    pub items: usize,
    pub passed: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BulkOutput {
    pub items: Vec<BulkItemResult>,
    pub summary: BulkSummary,
    pub version: String,
}

pub fn run_validate(
    name: Option<&str>,
    all: bool,
    changes: bool,
    specs: bool,
    item_type: Option<&str>,
    strict: bool,
    json: bool,
) -> Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    if all || changes || specs {
        return run_bulk_validation(&project_root, all || changes, all || specs, strict, json);
    }

    let name = name.ok_or_else(|| {
        OpenSpecError::Custom(
            "Nothing to validate. Try: --all, --changes, --specs, or <item-name>".to_string(),
        )
    })?;

    let available_changes = get_available_changes(&project_root)?;
    let available_specs = get_available_specs(&project_root)?;

    let is_change = available_changes.contains(&name.to_string());
    let is_spec = available_specs.contains(&name.to_string());

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
            available_changes.join(", "),
            available_specs.join(", ")
        )));
    }

    if item_type.is_none() && is_change && is_spec {
        return Err(OpenSpecError::Custom(format!(
            "Ambiguous item '{}' matches both a change and a spec. Use --type change|spec",
            name
        )));
    }

    let start = std::time::Instant::now();
    let (report, item_type_str) = match resolved_type {
        "change" => {
            let change_dir = project_root
                .join(OPENSPEC_DIR_NAME)
                .join("changes")
                .join(name);
            (validate_change(&change_dir, strict)?, "change")
        }
        "spec" => {
            let spec_path = project_root
                .join(OPENSPEC_DIR_NAME)
                .join("specs")
                .join(name)
                .join("spec.md");
            (validate_spec(&spec_path, strict)?, "spec")
        }
        _ => {
            return Err(OpenSpecError::Custom(format!(
                "Unknown type '{}'. Use 'change' or 'spec'",
                resolved_type
            )))
        }
    };
    let duration_ms = start.elapsed().as_millis() as u64;

    if json {
        let output = BulkOutput {
            items: vec![BulkItemResult {
                id: name.to_string(),
                item_type: item_type_str.to_string(),
                valid: report.valid,
                issues: report.issues,
                duration_ms,
            }],
            summary: BulkSummary {
                totals: BulkTotals {
                    items: 1,
                    passed: if report.valid { 1 } else { 0 },
                    failed: if report.valid { 0 } else { 1 },
                },
                by_type: {
                    let mut map = std::collections::HashMap::new();
                    map.insert(
                        item_type_str.to_string(),
                        BulkTotals {
                            items: 1,
                            passed: if report.valid { 1 } else { 0 },
                            failed: if report.valid { 0 } else { 1 },
                        },
                    );
                    map
                },
            },
            version: "1.0".to_string(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        print_report(name, item_type_str, &report);
    }

    if !report.valid {
        std::process::exit(1);
    }

    Ok(())
}

fn run_bulk_validation(
    project_root: &std::path::Path,
    validate_changes: bool,
    validate_specs: bool,
    strict: bool,
    json: bool,
) -> Result<()> {
    let change_ids = if validate_changes {
        get_available_changes(project_root)?
    } else {
        vec![]
    };
    let spec_ids = if validate_specs {
        get_available_specs(project_root)?
    } else {
        vec![]
    };

    if change_ids.is_empty() && spec_ids.is_empty() {
        if json {
            let output = BulkOutput {
                items: vec![],
                summary: BulkSummary {
                    totals: BulkTotals {
                        items: 0,
                        passed: 0,
                        failed: 0,
                    },
                    by_type: {
                        let mut map = std::collections::HashMap::new();
                        if validate_changes {
                            map.insert(
                                "change".to_string(),
                                BulkTotals {
                                    items: 0,
                                    passed: 0,
                                    failed: 0,
                                },
                            );
                        }
                        if validate_specs {
                            map.insert(
                                "spec".to_string(),
                                BulkTotals {
                                    items: 0,
                                    passed: 0,
                                    failed: 0,
                                },
                            );
                        }
                        map
                    },
                },
                version: "1.0".to_string(),
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("No items found to validate.");
        }
        return Ok(());
    }

    let mut results: Vec<BulkItemResult> = Vec::new();
    let mut passed = 0;
    let mut failed = 0;

    for id in &change_ids {
        let start = std::time::Instant::now();
        let change_dir = project_root
            .join(OPENSPEC_DIR_NAME)
            .join("changes")
            .join(id);
        let report = validate_change(&change_dir, strict)?;
        let duration_ms = start.elapsed().as_millis() as u64;
        let valid = report.valid;
        if valid {
            passed += 1;
        } else {
            failed += 1;
        }
        results.push(BulkItemResult {
            id: id.clone(),
            item_type: "change".to_string(),
            valid,
            issues: report.issues,
            duration_ms,
        });
    }

    for id in &spec_ids {
        let start = std::time::Instant::now();
        let spec_path = project_root
            .join(OPENSPEC_DIR_NAME)
            .join("specs")
            .join(id)
            .join("spec.md");
        let report = validate_spec(&spec_path, strict)?;
        let duration_ms = start.elapsed().as_millis() as u64;
        let valid = report.valid;
        if valid {
            passed += 1;
        } else {
            failed += 1;
        }
        results.push(BulkItemResult {
            id: id.clone(),
            item_type: "spec".to_string(),
            valid,
            issues: report.issues,
            duration_ms,
        });
    }

    results.sort_by(|a, b| a.id.cmp(&b.id));

    if json {
        let output = BulkOutput {
            items: results.clone(),
            summary: BulkSummary {
                totals: BulkTotals {
                    items: results.len(),
                    passed,
                    failed,
                },
                by_type: {
                    let mut map = std::collections::HashMap::new();
                    if validate_changes {
                        let change_results: Vec<_> =
                            results.iter().filter(|r| r.item_type == "change").collect();
                        let change_passed = change_results.iter().filter(|r| r.valid).count();
                        map.insert(
                            "change".to_string(),
                            BulkTotals {
                                items: change_results.len(),
                                passed: change_passed,
                                failed: change_results.len() - change_passed,
                            },
                        );
                    }
                    if validate_specs {
                        let spec_results: Vec<_> =
                            results.iter().filter(|r| r.item_type == "spec").collect();
                        let spec_passed = spec_results.iter().filter(|r| r.valid).count();
                        map.insert(
                            "spec".to_string(),
                            BulkTotals {
                                items: spec_results.len(),
                                passed: spec_passed,
                                failed: spec_results.len() - spec_passed,
                            },
                        );
                    }
                    map
                },
            },
            version: "1.0".to_string(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        for res in &results {
            if res.valid {
                println!("✓ {}/{}", res.item_type, res.id);
            } else {
                eprintln!("✗ {}/{}", res.item_type, res.id);
            }
        }
        println!(
            "Totals: {} passed, {} failed ({} items)",
            passed,
            failed,
            results.len()
        );
    }

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn validate_change(change_dir: &std::path::Path, _strict: bool) -> Result<ValidationReport> {
    let mut issues: Vec<ValidationIssue> = Vec::new();
    let specs_dir = change_dir.join("specs");

    if !specs_dir.exists() {
        issues.push(ValidationIssue {
            level: "ERROR".to_string(),
            path: "specs/".to_string(),
            message: "Change has no specs/ directory. Add delta specs with ADDED/MODIFIED/REMOVED/RENAMED sections.".to_string(),
        });
        return Ok(ValidationReport {
            valid: false,
            issues,
        });
    }

    let mut total_deltas = 0;
    let mut found_specs = false;

    if let Ok(entries) = std::fs::read_dir(&specs_dir) {
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let spec_name = entry.file_name().to_string_lossy().to_string();
            let spec_file = specs_dir.join(&spec_name).join("spec.md");

            if !spec_file.exists() {
                continue;
            }

            found_specs = true;
            let content = std::fs::read_to_string(&spec_file)
                .map_err(|e| OpenSpecError::Custom(format!("Failed to read spec: {}", e)))?;

            let plan = parse_delta_spec(&content);
            let entry_path = format!("{}/spec.md", spec_name);

            let has_entries = !plan.added.is_empty()
                || !plan.modified.is_empty()
                || !plan.removed.is_empty()
                || !plan.renamed.is_empty();

            if !has_entries {
                if plan.section_presence.added
                    || plan.section_presence.modified
                    || plan.section_presence.removed
                    || plan.section_presence.renamed
                {
                    issues.push(ValidationIssue {
                        level: "ERROR".to_string(),
                        path: entry_path.clone(),
                        message: "Delta sections found but no requirement entries parsed. Ensure each section has '### Requirement:' blocks.".to_string(),
                    });
                } else {
                    issues.push(ValidationIssue {
                        level: "ERROR".to_string(),
                        path: entry_path.clone(),
                        message:
                            "No delta sections found. Add headers like '## ADDED Requirements'."
                                .to_string(),
                    });
                }
            }

            for req in &plan.added {
                total_deltas += 1;
                if !contains_shall_or_must(&req.raw) {
                    issues.push(ValidationIssue {
                        level: "ERROR".to_string(),
                        path: entry_path.clone(),
                        message: format!("ADDED \"{}\" must contain SHALL or MUST", req.name),
                    });
                }
                if count_scenarios(&req.raw) < 1 {
                    issues.push(ValidationIssue {
                        level: "ERROR".to_string(),
                        path: entry_path.clone(),
                        message: format!(
                            "ADDED \"{}\" must include at least one scenario",
                            req.name
                        ),
                    });
                }
            }

            for req in &plan.modified {
                total_deltas += 1;
                if !contains_shall_or_must(&req.raw) {
                    issues.push(ValidationIssue {
                        level: "ERROR".to_string(),
                        path: entry_path.clone(),
                        message: format!("MODIFIED \"{}\" must contain SHALL or MUST", req.name),
                    });
                }
                if count_scenarios(&req.raw) < 1 {
                    issues.push(ValidationIssue {
                        level: "ERROR".to_string(),
                        path: entry_path.clone(),
                        message: format!(
                            "MODIFIED \"{}\" must include at least one scenario",
                            req.name
                        ),
                    });
                }
            }

            total_deltas += plan.removed.len();
            total_deltas += plan.renamed.len();
        }
    }

    if !found_specs {
        issues.push(ValidationIssue {
            level: "ERROR".to_string(),
            path: "specs/".to_string(),
            message:
                "Change has no spec files. Add specs/<capability>/spec.md with delta sections."
                    .to_string(),
        });
    } else if total_deltas == 0 {
        issues.push(ValidationIssue {
            level: "ERROR".to_string(),
            path: "file".to_string(),
            message:
                "Change has no deltas. Add requirements to ADDED/MODIFIED/REMOVED/RENAMED sections."
                    .to_string(),
        });
    }

    let valid = issues.iter().all(|i| i.level != "ERROR");
    Ok(ValidationReport { valid, issues })
}

fn validate_spec(spec_path: &std::path::Path, _strict: bool) -> Result<ValidationReport> {
    let mut issues: Vec<ValidationIssue> = Vec::new();

    if !spec_path.exists() {
        issues.push(ValidationIssue {
            level: "ERROR".to_string(),
            path: "file".to_string(),
            message: format!("Spec file not found: {}", spec_path.display()),
        });
        return Ok(ValidationReport {
            valid: false,
            issues,
        });
    }

    let content = std::fs::read_to_string(spec_path)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to read spec: {}", e)))?;

    let spec_name = spec_path
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut parser = SpecParser::new(&content);
    match parser.parse_spec(&spec_name) {
        Ok(spec) => {
            if spec.overview.len() < 10 {
                issues.push(ValidationIssue {
                    level: "WARNING".to_string(),
                    path: "overview".to_string(),
                    message: "Purpose section is too brief. Provide more context.".to_string(),
                });
            }

            for (idx, req) in spec.requirements.iter().enumerate() {
                if req.scenarios.is_empty() {
                    issues.push(ValidationIssue {
                        level: "WARNING".to_string(),
                        path: format!("requirements[{}].scenarios", idx),
                        message: format!(
                            "Requirement '{}' has no scenarios. Add '#### Scenario:' blocks.",
                            req.text.chars().take(50).collect::<String>()
                        ),
                    });
                }
            }
        }
        Err(e) => {
            issues.push(ValidationIssue {
                level: "ERROR".to_string(),
                path: "file".to_string(),
                message: e,
            });
        }
    }

    let valid = issues.iter().all(|i| i.level != "ERROR");
    Ok(ValidationReport { valid, issues })
}

fn print_report(name: &str, item_type: &str, report: &ValidationReport) {
    if report.valid {
        println!(
            "{} '{}' is valid",
            if item_type == "change" {
                "Change"
            } else {
                "Specification"
            },
            name
        );
    } else {
        eprintln!(
            "{} '{}' has issues",
            if item_type == "change" {
                "Change"
            } else {
                "Specification"
            },
            name
        );
        for issue in &report.issues {
            let prefix = match issue.level.as_str() {
                "ERROR" => "✗",
                "WARNING" => "⚠",
                _ => "ℹ",
            };
            eprintln!(
                "{} [{}] {}: {}",
                prefix, issue.level, issue.path, issue.message
            );
        }

        eprintln!();
        eprintln!("Next steps:");
        if item_type == "change" {
            eprintln!("  - Ensure change has deltas in specs/: use headers ## ADDED/MODIFIED/REMOVED/RENAMED Requirements");
            eprintln!("  - Each requirement MUST include at least one #### Scenario: block");
            eprintln!("  - Debug parsed deltas: openspec show <id> --json --deltas-only");
        } else {
            eprintln!("  - Ensure spec includes ## Purpose and ## Requirements sections");
            eprintln!("  - Each requirement MUST include at least one #### Scenario: block");
        }
    }
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
                let proposal_path = changes_dir.join(&name).join("proposal.md");
                if proposal_path.exists() {
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

fn contains_shall_or_must(text: &str) -> bool {
    text.to_uppercase().contains("SHALL") || text.to_uppercase().contains("MUST")
}

fn count_scenarios(block_raw: &str) -> usize {
    block_raw
        .lines()
        .filter(|line| line.trim().starts_with("#### "))
        .count()
}
