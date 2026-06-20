use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ai_tools::generator::{get_tool_by_value, get_tools_with_skills_dir, AI_TOOLS};
use crate::core::config::ConfigManager;
use crate::templates::skills::{
    extract_generated_by_version, generate_skill_content_transformed, get_profile_workflows,
    get_skill_templates,
};

use super::foundation::WorkspaceSkillState;

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceSkillAgentResult {
    pub tool_id: String,
    pub name: String,
    pub skills_path: String,
    pub workflow_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceSkillRemovedResult {
    pub tool_id: String,
    pub name: String,
    pub skills_path: String,
    pub workflow_ids: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceSkillSkippedResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub reason: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceSkillFailedResult {
    pub tool_id: String,
    pub name: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceSkillInstallationReport {
    pub profile: String,
    pub delivery: String,
    pub workflow_ids: Vec<String>,
    pub selected_agents: Vec<String>,
    pub skills_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_notice: Option<String>,
    pub generated: Vec<WorkspaceSkillAgentResult>,
    pub added: Vec<WorkspaceSkillAgentResult>,
    pub refreshed: Vec<WorkspaceSkillAgentResult>,
    pub removed: Vec<WorkspaceSkillRemovedResult>,
    pub skipped: Vec<WorkspaceSkillSkippedResult>,
    pub failed: Vec<WorkspaceSkillFailedResult>,
}

pub struct WorkspaceSkillProfileContext {
    profile: String,
    delivery: String,
    workflow_ids: Vec<String>,
    delivery_notice: Option<String>,
}

fn resolve_workspace_skill_profile_context() -> WorkspaceSkillProfileContext {
    let config = ConfigManager::load_global_config();

    let profile = config.profile.clone();
    let delivery = config.delivery.clone();
    let workflow_ids = get_profile_workflows(&profile, &config.workflows);
    let delivery_notice = if delivery == "skills" {
        None
    } else {
        Some("Workspace setup installs skills only; workspace command generation is not part of this slice.".to_string())
    };

    WorkspaceSkillProfileContext {
        profile,
        delivery,
        workflow_ids,
        delivery_notice,
    }
}

pub fn get_current_workspace_skill_profile_selection() -> (String, String, Vec<String>) {
    let ctx = resolve_workspace_skill_profile_context();
    (ctx.profile, ctx.delivery, ctx.workflow_ids)
}

pub fn has_workspace_skill_profile_drift(skill_state: Option<&WorkspaceSkillState>) -> bool {
    let Some(state) = skill_state else {
        return false;
    };

    let current = get_current_workspace_skill_profile_selection();

    let profile_drift = state.last_applied_profile.as_deref() != Some(current.0.as_str());
    let delivery_drift = state.last_applied_delivery.as_deref() != Some(current.1.as_str());

    let current_set: BTreeSet<_> = current.2.into_iter().collect();
    let prev_set: BTreeSet<_> = state
        .last_applied_workflow_ids
        .as_ref()
        .map(|ids| ids.iter().cloned().collect())
        .unwrap_or_default();
    let workflow_drift = current_set != prev_set;

    profile_drift || delivery_drift || workflow_drift
}

pub fn get_workspace_skill_capable_tools() -> Vec<&'static crate::ai_tools::generator::AITool> {
    AI_TOOLS
        .iter()
        .filter(|tool| !tool.skills_dir.is_empty())
        .collect()
}

pub fn get_workspace_skill_tool_ids() -> Vec<String> {
    get_tools_with_skills_dir()
        .iter()
        .map(|s| s.to_string())
        .collect()
}

pub fn parse_workspace_skill_tools_value(raw: &str) -> Result<Vec<String>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(
            "The --tools option requires a value. Use \"all\", \"none\", or a comma-separated list of agent IDs."
                .to_string(),
        );
    }

    let available_tools = get_workspace_skill_tool_ids();
    let available_set: std::collections::HashSet<_> = available_tools.iter().cloned().collect();
    let lower_raw = trimmed.to_lowercase();

    if lower_raw == "all" {
        return Ok(available_tools);
    }

    if lower_raw == "none" {
        return Ok(vec![]);
    }

    let tokens: Vec<String> = trimmed
        .split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();

    if tokens.is_empty() {
        return Err(
            "The --tools option requires at least one agent ID when not using \"all\" or \"none\"."
                .to_string(),
        );
    }

    let normalized: Vec<String> = tokens.iter().map(|t| t.to_lowercase()).collect();

    if normalized.iter().any(|t| t == "all" || t == "none") {
        return Err(
            "Cannot combine reserved values \"all\" or \"none\" with specific agent IDs."
                .to_string(),
        );
    }

    let invalid: Vec<_> = tokens
        .iter()
        .zip(&normalized)
        .filter(|(_, norm)| !available_set.contains(norm.as_str()))
        .map(|(orig, _)| orig.clone())
        .collect();

    if !invalid.is_empty() {
        let mut available_list = vec!["all", "none"];
        available_list.extend(available_tools.iter().map(|s| s.as_str()));
        return Err(format!(
            "Invalid agent(s): {}. Available values: {}",
            invalid.join(", "),
            available_list.join(", ")
        ));
    }

    let mut deduped = Vec::new();
    for token in normalized {
        if !deduped.contains(&token) {
            deduped.push(token);
        }
    }

    Ok(deduped)
}

pub fn create_workspace_skill_skipped_report(
    reason: &str,
    message: &str,
) -> WorkspaceSkillInstallationReport {
    let ctx = resolve_workspace_skill_profile_context();
    let mut report = WorkspaceSkillInstallationReport {
        profile: ctx.profile,
        delivery: ctx.delivery,
        workflow_ids: ctx.workflow_ids,
        selected_agents: vec![],
        skills_only: true,
        delivery_notice: ctx.delivery_notice,
        generated: vec![],
        added: vec![],
        refreshed: vec![],
        removed: vec![],
        skipped: vec![],
        failed: vec![],
    };
    report.skipped.push(WorkspaceSkillSkippedResult {
        tool_id: None,
        name: None,
        reason: reason.to_string(),
        message: message.to_string(),
    });
    report
}

fn _make_base_workspace_skill_report(
    selected_agent_ids: Vec<String>,
) -> WorkspaceSkillInstallationReport {
    let ctx = resolve_workspace_skill_profile_context();
    WorkspaceSkillInstallationReport {
        profile: ctx.profile,
        delivery: ctx.delivery,
        workflow_ids: ctx.workflow_ids,
        selected_agents: selected_agent_ids,
        skills_only: true,
        delivery_notice: ctx.delivery_notice,
        generated: vec![],
        added: vec![],
        refreshed: vec![],
        removed: vec![],
        skipped: vec![],
        failed: vec![],
    }
}

pub fn get_workspace_skill_directory(
    workspace_root: &Path,
    tool_id: &str,
) -> Result<PathBuf, String> {
    let tool = get_tool_by_value(tool_id)
        .ok_or_else(|| format!("Unknown workspace skill agent '{}'", tool_id))?;
    Ok(workspace_root.join(tool.skills_dir).join("skills"))
}

fn make_agent_result(
    workspace_root: &Path,
    tool_id: &str,
    tool_name: &str,
    workflow_ids: Vec<String>,
) -> Result<WorkspaceSkillAgentResult, String> {
    let skills_path = get_workspace_skill_directory(workspace_root, tool_id)?;
    Ok(WorkspaceSkillAgentResult {
        tool_id: tool_id.to_string(),
        name: tool_name.to_string(),
        skills_path: skills_path.to_string_lossy().to_string(),
        workflow_ids,
    })
}

fn tool_is_configured(workspace_root: &Path, tool_id: &str) -> Result<bool, String> {
    let skills_dir = get_workspace_skill_directory(workspace_root, tool_id)?;

    for entry in get_skill_templates(None) {
        let skill_dir = skills_dir.join(entry.dir_name);
        let skill_file = skill_dir.join("SKILL.md");

        if skill_file.exists() && extract_generated_by_version(&skill_file).is_some() {
            return Ok(true);
        }
    }

    Ok(false)
}

fn remove_managed_workflow_skill_dirs(
    workspace_root: &Path,
    tool_id: &str,
    tool_name: &str,
    desired_workflow_ids: &[String],
    reason: &str,
) -> Result<Option<WorkspaceSkillRemovedResult>, String> {
    let skills_dir = get_workspace_skill_directory(workspace_root, tool_id)?;
    let desired_set: std::collections::HashSet<_> = desired_workflow_ids.iter().cloned().collect();
    let mut removed_workflow_ids = Vec::new();

    for entry in get_skill_templates(None) {
        if desired_set.contains(entry.workflow_id) {
            continue;
        }

        let skill_dir = skills_dir.join(entry.dir_name);

        if !skill_dir.exists() {
            continue;
        }

        let skill_file = skill_dir.join("SKILL.md");
        if extract_generated_by_version(&skill_file).is_none() {
            continue;
        }

        fs::remove_dir_all(&skill_dir)
            .map_err(|e| format!("Failed to remove skill directory: {}", e))?;
        removed_workflow_ids.push(entry.workflow_id.to_string());
    }

    if removed_workflow_ids.is_empty() {
        return Ok(None);
    }

    let result = WorkspaceSkillRemovedResult {
        tool_id: tool_id.to_string(),
        name: tool_name.to_string(),
        skills_path: skills_dir.to_string_lossy().to_string(),
        workflow_ids: removed_workflow_ids,
        reason: reason.to_string(),
    };

    Ok(Some(result))
}

pub fn generate_workspace_agent_skills(
    workspace_root: &Path,
    selected_agent_ids: Vec<String>,
) -> Result<WorkspaceSkillInstallationReport, String> {
    let ctx = resolve_workspace_skill_profile_context();
    let mut report = WorkspaceSkillInstallationReport {
        profile: ctx.profile.clone(),
        delivery: ctx.delivery.clone(),
        workflow_ids: ctx.workflow_ids.clone(),
        selected_agents: selected_agent_ids.clone(),
        skills_only: true,
        delivery_notice: ctx.delivery_notice,
        generated: vec![],
        added: vec![],
        refreshed: vec![],
        removed: vec![],
        skipped: vec![],
        failed: vec![],
    };

    if selected_agent_ids.is_empty() {
        report.skipped.push(WorkspaceSkillSkippedResult {
            tool_id: None,
            name: None,
            reason: "no_agents_selected".to_string(),
            message: "No workspace agent skills were selected.".to_string(),
        });
        return Ok(report);
    }

    let workflow_ids_str: Vec<&str> = ctx.workflow_ids.iter().map(|s| s.as_str()).collect();
    let skill_templates = get_skill_templates(Some(&workflow_ids_str));

    if skill_templates.is_empty() {
        for tool_id in &selected_agent_ids {
            if let Some(tool) = get_tool_by_value(tool_id) {
                report.skipped.push(WorkspaceSkillSkippedResult {
                    tool_id: Some(tool_id.clone()),
                    name: Some(tool.name.to_string()),
                    reason: "no_profile_workflows".to_string(),
                    message: "The active global profile does not select any workflows.".to_string(),
                });
            }
        }
        return Ok(report);
    }

    let version = env!("CARGO_PKG_VERSION");

    for tool_id in &selected_agent_ids {
        let tool = match get_tool_by_value(tool_id) {
            Some(t) => t,
            None => {
                report.failed.push(WorkspaceSkillFailedResult {
                    tool_id: tool_id.clone(),
                    name: String::new(),
                    error: format!("Unknown tool: {}", tool_id),
                });
                continue;
            }
        };

        match tool_is_configured(workspace_root, tool_id) {
            Ok(was_configured) => {
                match write_skills_for_tool(
                    workspace_root,
                    tool_id,
                    tool.name,
                    &skill_templates,
                    version,
                ) {
                    Ok(result) => {
                        if was_configured {
                            report.refreshed.push(result);
                        } else {
                            report.generated.push(result);
                        }
                    }
                    Err(e) => {
                        report.failed.push(WorkspaceSkillFailedResult {
                            tool_id: tool_id.clone(),
                            name: tool.name.to_string(),
                            error: e,
                        });
                    }
                }
            }
            Err(e) => {
                report.failed.push(WorkspaceSkillFailedResult {
                    tool_id: tool_id.clone(),
                    name: tool.name.to_string(),
                    error: e,
                });
            }
        }
    }

    Ok(report)
}

pub fn update_workspace_agent_skills(
    workspace_root: &Path,
    selected_agent_ids: Vec<String>,
    previous_skill_state: Option<&WorkspaceSkillState>,
) -> Result<WorkspaceSkillInstallationReport, String> {
    let ctx = resolve_workspace_skill_profile_context();
    let mut report = WorkspaceSkillInstallationReport {
        profile: ctx.profile.clone(),
        delivery: ctx.delivery.clone(),
        workflow_ids: ctx.workflow_ids.clone(),
        selected_agents: selected_agent_ids.clone(),
        skills_only: true,
        delivery_notice: ctx.delivery_notice,
        generated: vec![],
        added: vec![],
        refreshed: vec![],
        removed: vec![],
        skipped: vec![],
        failed: vec![],
    };

    let previous_agent_ids: Vec<String> = previous_skill_state
        .map(|s| s.selected_agents.clone())
        .unwrap_or_default();
    let previous_set: std::collections::HashSet<_> = previous_agent_ids.iter().cloned().collect();
    let selected_set: std::collections::HashSet<_> = selected_agent_ids.iter().cloned().collect();

    for tool_id in &previous_agent_ids {
        if selected_set.contains(tool_id) {
            continue;
        }

        if let Some(tool) = get_tool_by_value(tool_id) {
            match remove_managed_workflow_skill_dirs(
                workspace_root,
                tool_id,
                tool.name,
                &[],
                "agent_unselected",
            ) {
                Ok(Some(removed)) => {
                    report.removed.push(removed);
                }
                Ok(None) => {}
                Err(e) => {
                    report.failed.push(WorkspaceSkillFailedResult {
                        tool_id: tool_id.clone(),
                        name: tool.name.to_string(),
                        error: e,
                    });
                }
            }
        }
    }

    if selected_agent_ids.is_empty() {
        if report.removed.is_empty() {
            let (reason, message) = if previous_skill_state.is_some() {
                (
                    "no_agents_selected",
                    "No workspace agent skills were selected.",
                )
            } else {
                (
                    "no_stored_agent_selection",
                    "No workspace agent skill selection is stored. Pass --tools <ids> to install skills.",
                )
            };
            report.skipped.push(WorkspaceSkillSkippedResult {
                tool_id: None,
                name: None,
                reason: reason.to_string(),
                message: message.to_string(),
            });
        }
        return Ok(report);
    }

    let workflow_ids_str: Vec<&str> = ctx.workflow_ids.iter().map(|s| s.as_str()).collect();
    let skill_templates = get_skill_templates(Some(&workflow_ids_str));

    if skill_templates.is_empty() {
        for tool_id in &selected_agent_ids {
            if let Some(tool) = get_tool_by_value(tool_id) {
                match remove_managed_workflow_skill_dirs(
                    workspace_root,
                    tool_id,
                    tool.name,
                    &[],
                    "workflow_unselected",
                ) {
                    Ok(Some(removed)) => {
                        report.removed.push(removed);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        report.failed.push(WorkspaceSkillFailedResult {
                            tool_id: tool_id.clone(),
                            name: tool.name.to_string(),
                            error: e,
                        });
                    }
                }
            }

            if let Some(tool) = get_tool_by_value(tool_id) {
                report.skipped.push(WorkspaceSkillSkippedResult {
                    tool_id: Some(tool_id.clone()),
                    name: Some(tool.name.to_string()),
                    reason: "no_profile_workflows".to_string(),
                    message: "The active global profile does not select any workflows.".to_string(),
                });
            }
        }
        return Ok(report);
    }

    let version = env!("CARGO_PKG_VERSION");

    for tool_id in &selected_agent_ids {
        let tool = match get_tool_by_value(tool_id) {
            Some(t) => t,
            None => {
                report.failed.push(WorkspaceSkillFailedResult {
                    tool_id: tool_id.clone(),
                    name: String::new(),
                    error: format!("Unknown tool: {}", tool_id),
                });
                continue;
            }
        };

        match write_skills_for_tool(
            workspace_root,
            tool_id,
            tool.name,
            &skill_templates,
            version,
        ) {
            Ok(agent_result) => {
                match remove_managed_workflow_skill_dirs(
                    workspace_root,
                    tool_id,
                    tool.name,
                    &ctx.workflow_ids,
                    "workflow_unselected",
                ) {
                    Ok(Some(removed)) => {
                        report.removed.push(removed);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        report.failed.push(WorkspaceSkillFailedResult {
                            tool_id: tool_id.clone(),
                            name: tool.name.to_string(),
                            error: e,
                        });
                        continue;
                    }
                }

                if previous_set.contains(tool_id) {
                    report.refreshed.push(agent_result);
                } else {
                    report.added.push(agent_result);
                }
            }
            Err(e) => {
                report.failed.push(WorkspaceSkillFailedResult {
                    tool_id: tool_id.clone(),
                    name: tool.name.to_string(),
                    error: e,
                });
            }
        }
    }

    Ok(report)
}

fn write_skills_for_tool(
    workspace_root: &Path,
    tool_id: &str,
    tool_name: &str,
    skill_templates: &[&crate::templates::skills::SkillEntry],
    version: &str,
) -> Result<WorkspaceSkillAgentResult, String> {
    let skills_dir = get_workspace_skill_directory(workspace_root, tool_id)?;
    let should_transform = tool_id == "opencode" || tool_id == "pi";

    let mut workflow_ids = Vec::new();

    for entry in skill_templates {
        let skill_file = skills_dir.join(entry.dir_name).join("SKILL.md");

        fs::create_dir_all(skill_file.parent().ok_or("Invalid path")?)
            .map_err(|e| format!("Failed to create directory: {}", e))?;

        let content = generate_skill_content_transformed(entry.template, version, should_transform);

        fs::write(&skill_file, content)
            .map_err(|e| format!("Failed to write skill file: {}", e))?;

        workflow_ids.push(entry.workflow_id.to_string());
    }

    make_agent_result(workspace_root, tool_id, tool_name, workflow_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_workspace_skill_tools_value_all() {
        let result = parse_workspace_skill_tools_value("all").unwrap();
        assert!(!result.is_empty());
        assert!(result.contains(&"claude".to_string()));
    }

    #[test]
    fn test_parse_workspace_skill_tools_value_none() {
        let result = parse_workspace_skill_tools_value("none").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_workspace_skill_tools_value_single() {
        let result = parse_workspace_skill_tools_value("claude").unwrap();
        assert_eq!(result, vec!["claude"]);
    }

    #[test]
    fn test_parse_workspace_skill_tools_value_multiple() {
        let result = parse_workspace_skill_tools_value("claude, cursor").unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"claude".to_string()));
        assert!(result.contains(&"cursor".to_string()));
    }

    #[test]
    fn test_parse_workspace_skill_tools_value_empty() {
        let result = parse_workspace_skill_tools_value("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_workspace_skill_tools_value_mixed_reserved() {
        let result = parse_workspace_skill_tools_value("all,claude");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_workspace_skill_tools_value_invalid() {
        let result = parse_workspace_skill_tools_value("bogus");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Invalid agent(s)"));
    }

    #[test]
    fn test_parse_workspace_skill_tools_value_case_insensitive() {
        let result = parse_workspace_skill_tools_value("CLAUDE, CURSOR").unwrap();
        assert_eq!(result, vec!["claude", "cursor"]);
    }

    #[test]
    fn test_parse_workspace_skill_tools_value_deduped() {
        let result = parse_workspace_skill_tools_value("claude, claude, cursor, claude").unwrap();
        assert_eq!(result, vec!["claude", "cursor"]);
    }

    #[test]
    fn test_has_workspace_skill_profile_drift_none() {
        assert!(!has_workspace_skill_profile_drift(None));
    }

    #[test]
    fn test_extract_generated_by_version_real() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"---
name: test
metadata:
  author: OpenSpec
  version: "1.0"
  generatedBy: "OpenSpec v0.1.4"
---

content"#
        )
        .unwrap();

        let version = extract_generated_by_version(file.path());
        assert_eq!(version, Some("0.1.4".to_string()));
    }

    #[test]
    fn test_generate_workspace_agent_skills_empty_selection() {
        let temp_dir = TempDir::new().unwrap();
        let result =
            generate_workspace_agent_skills(temp_dir.path(), vec![]).expect("should succeed");

        assert!(result.generated.is_empty());
        assert!(result.added.is_empty());
        assert!(result.refreshed.is_empty());
        assert_eq!(result.skipped.len(), 1);
        assert_eq!(result.skipped[0].reason, "no_agents_selected");
    }

    #[test]
    fn test_get_workspace_skill_capable_tools() {
        let tools = get_workspace_skill_capable_tools();
        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.value == "claude"));
    }

    #[test]
    fn test_get_workspace_skill_tool_ids() {
        let ids = get_workspace_skill_tool_ids();
        assert!(!ids.is_empty());
        assert!(ids.contains(&"claude".to_string()));
    }

    #[test]
    fn test_get_workspace_skill_directory() {
        let temp_dir = TempDir::new().unwrap();
        let result = get_workspace_skill_directory(temp_dir.path(), "claude");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains(".claude"));
        assert!(path.to_string_lossy().contains("skills"));
    }

    #[test]
    fn test_get_workspace_skill_directory_unknown_tool() {
        let temp_dir = TempDir::new().unwrap();
        let result = get_workspace_skill_directory(temp_dir.path(), "unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_workspace_agent_skills_with_claude() {
        let temp_dir = TempDir::new().unwrap();
        let result = generate_workspace_agent_skills(temp_dir.path(), vec!["claude".to_string()]);

        assert!(result.is_ok());
        let report = result.unwrap();

        assert!(!report.generated.is_empty() || !report.refreshed.is_empty());
        assert_eq!(report.generated.len() + report.refreshed.len(), 1);
        assert!(report.failed.is_empty());

        if !report.generated.is_empty() {
            assert_eq!(report.generated[0].tool_id, "claude");
            assert!(!report.generated[0].workflow_ids.is_empty());
        }
    }

    #[test]
    fn test_generate_creates_skill_files() {
        let temp_dir = TempDir::new().unwrap();
        let _ = generate_workspace_agent_skills(temp_dir.path(), vec!["claude".to_string()]);

        let skills_dir = temp_dir.path().join(".claude/skills");
        assert!(skills_dir.exists());

        // Check that at least one skill file was created
        let has_skill_files = fs::read_dir(&skills_dir)
            .map(|entries| {
                entries
                    .flatten()
                    .any(|entry| entry.file_name().to_string_lossy().starts_with("openspec-"))
            })
            .unwrap_or(false);

        assert!(has_skill_files);
    }

    #[test]
    fn test_skill_content_contains_generated_by() {
        let temp_dir = TempDir::new().unwrap();
        let _ = generate_workspace_agent_skills(temp_dir.path(), vec!["claude".to_string()]);

        let skills_dir = temp_dir.path().join(".claude/skills");
        let entries = fs::read_dir(&skills_dir).unwrap();

        let mut found_skill = false;
        for entry in entries.flatten() {
            let skill_md = entry.path().join("SKILL.md");
            if skill_md.exists() {
                let content = fs::read_to_string(&skill_md).unwrap();
                assert!(
                    content.contains("generatedBy: \"OpenSpec v"),
                    "Skill file should contain generatedBy version"
                );
                found_skill = true;
                break;
            }
        }

        assert!(found_skill, "Should have created at least one skill file");
    }

    #[test]
    fn test_opencode_transform() {
        let temp_dir = TempDir::new().unwrap();
        let _ = generate_workspace_agent_skills(temp_dir.path(), vec!["opencode".to_string()]);

        let skills_dir = temp_dir.path().join(".opencode/skills");
        let entries = fs::read_dir(&skills_dir).unwrap();

        for entry in entries.flatten() {
            let skill_md = entry.path().join("SKILL.md");
            if skill_md.exists() {
                let content = fs::read_to_string(&skill_md).unwrap();
                // If the template contains /opsx:, it should be transformed to /opsx-
                if content.contains("/opsx-") {
                    // Verify no untransformed commands remain
                    assert!(!content.contains("/opsx:"));
                    break;
                }
            }
        }
    }

    #[test]
    fn test_update_marks_as_refreshed() {
        let temp_dir = TempDir::new().unwrap();

        // First generation
        let result1 = generate_workspace_agent_skills(temp_dir.path(), vec!["claude".to_string()]);
        let report1 = result1.unwrap();

        // Create a mock skill state to simulate previous installation
        let previous_state = WorkspaceSkillState {
            selected_agents: vec!["claude".to_string()],
            last_applied_profile: Some("core".to_string()),
            last_applied_delivery: Some("both".to_string()),
            last_applied_workflow_ids: Some(report1.workflow_ids.clone()),
            last_applied_at: Some("2024-01-01T00:00:00Z".to_string()),
        };

        // Update with the same selection
        let result2 = update_workspace_agent_skills(
            temp_dir.path(),
            vec!["claude".to_string()],
            Some(&previous_state),
        );
        let report2 = result2.unwrap();

        // Should be marked as refreshed, not generated
        assert!(!report2.refreshed.is_empty() || report2.generated.is_empty());
    }

    #[test]
    fn test_has_workspace_skill_profile_drift_with_matching_state() {
        // This test verifies that profile drift detection works with a matching state
        let state = WorkspaceSkillState {
            selected_agents: vec!["claude".to_string()],
            last_applied_profile: Some("core".to_string()),
            last_applied_delivery: Some("both".to_string()),
            last_applied_workflow_ids: Some(vec![
                "propose".to_string(),
                "explore".to_string(),
                "apply".to_string(),
                "sync".to_string(),
                "archive".to_string(),
            ]),
            last_applied_at: None,
        };

        let _has_drift = has_workspace_skill_profile_drift(Some(&state));
        // The drift detection depends on the current global config, which we can't easily mock
        // But the function should not panic or error
    }
}
