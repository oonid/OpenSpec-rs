use crate::cli::args::WorkspaceCommands;
use crate::core::workspace::{
    add_workspace_link, resolve_selected_workspace, WorkspaceStatus,
    has_workspace_skill_profile_drift, is_workspace_root, list_workspace_registry_entries,
    load_workspace_registry, read_workspace_view_state, update_workspace_link,
};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
struct WorkspaceListItem {
    name: String,
    root: String,
    context: Option<String>,
    links: Vec<WorkspaceLinkInfo>,
    status: Vec<WorkspaceStatus>,
}

#[derive(Debug, Clone, Serialize)]
struct WorkspaceLinkInfo {
    name: String,
    path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct WorkspaceListOutput {
    workspaces: Vec<WorkspaceListItem>,
}

#[derive(Debug, Clone, Serialize)]
struct WorkspaceMutationOutput {
    workspace: WorkspaceOutput,
    link: WorkspaceLinkOutput,
}

#[derive(Debug, Clone, Serialize)]
struct WorkspaceOutput {
    name: String,
    root: String,
    context: Option<String>,
    links: Vec<WorkspaceLinkInfo>,
}

#[derive(Debug, Clone, Serialize)]
struct WorkspaceLinkOutput {
    name: String,
    path: String,
}

#[derive(Debug, Clone, Serialize)]
struct WorkspaceDoctorOutput {
    workspace: WorkspaceDoctorWorkspaceInfo,
    status: Vec<WorkspaceStatus>,
}

#[derive(Debug, Clone, Serialize)]
struct WorkspaceDoctorWorkspaceInfo {
    name: String,
    root: String,
    state_path: String,
    planning_path: String,
    context: Option<String>,
    links: Vec<WorkspaceDoctorLink>,
}

#[derive(Debug, Clone, Serialize)]
struct WorkspaceDoctorLink {
    name: String,
    path: Option<String>,
    exists: bool,
}

pub fn run(cmd: WorkspaceCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        WorkspaceCommands::List { json } => run_list(json),
        WorkspaceCommands::Link {
            name_or_path,
            path,
            workspace,
            json,
        } => run_link(name_or_path.as_deref(), path.as_deref(), workspace.as_deref(), json),
        WorkspaceCommands::Relink {
            name,
            path,
            workspace,
            json,
        } => run_relink(&name, &path, workspace.as_deref(), json),
        WorkspaceCommands::Doctor { workspace, json } => run_doctor(workspace.as_deref(), json),
    }
}

fn run_list(json: bool) -> Result<(), Box<dyn std::error::Error>> {

    let registry = load_workspace_registry(None)
        .map_err(|e| format!("Failed to load workspace registry: {}", e))?;

    let entries = list_workspace_registry_entries(&registry);

    let mut workspaces = Vec::new();

    for entry in entries {
        let mut status = Vec::new();
        let mut links = Vec::new();
        let mut context = None;

        let root_str = entry.workspace_root.clone();

        // Check if the workspace root exists and is valid
        let root_path = Path::new(&root_str);
        if !root_path.exists() || !is_workspace_root(root_path) {
            status.push(WorkspaceStatus::error(
                "workspace_root_missing",
                "Workspace location does not exist.",
            ));
            workspaces.push(WorkspaceListItem {
                name: entry.name.clone(),
                root: root_str,
                context,
                links,
                status,
            });
            continue;
        }

        // Try to read the view state
        match read_workspace_view_state(root_path) {
            Ok(view_state) => {
                // Extract context
                if view_state.context.is_some() {
                    context = Some("initiative".to_string());
                }

                // Build links list
                for (link_name, link_path) in &view_state.links {
                    links.push(WorkspaceLinkInfo {
                        name: link_name.clone(),
                        path: link_path.as_ref().map(|p| p.to_string()),
                    });
                }
                links.sort_by(|a, b| a.name.cmp(&b.name));

                // Check for skill drift
                if has_workspace_skill_profile_drift(view_state.workspace_skills.as_ref()) {
                    status.push(WorkspaceStatus::warning(
                        "workspace_skills_out_of_sync",
                        "Workspace-local agent skills are out of sync with the active global profile.",
                    ));
                }
            }
            Err(e) => {
                status.push(WorkspaceStatus::error(
                    "workspace_state_invalid",
                    &format!("Workspace state could not be read: {}", e),
                ));
            }
        }

        workspaces.push(WorkspaceListItem {
            name: entry.name,
            root: root_str,
            context,
            links,
            status,
        });
    }

    if json {
        let output = WorkspaceListOutput { workspaces };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        if workspaces.is_empty() {
            println!("No workspaces registered.");
            println!("Use: openspec workspace link <path> to add a workspace");
        } else {
            for ws in &workspaces {
                println!("\n{}", ws.name);
                println!("  root: {}", ws.root);
                for link in &ws.links {
                    if let Some(path) = &link.path {
                        println!("  {} -> {}", link.name, path);
                    } else {
                        println!("  {} -> (no local path recorded)", link.name);
                    }
                }
                for s in &ws.status {
                    println!("  [{}] {}: {}", s.level, s.code, s.message);
                }
            }
        }
    }

    Ok(())
}

fn run_link(
    name_or_path: Option<&str>,
    path: Option<&str>,
    workspace: Option<&str>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;

    // Resolve the workspace
    let selected = resolve_selected_workspace(workspace, &cwd, None)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    // Validate that we have at least one argument
    let name_or_path_val = name_or_path.ok_or_else(|| {
        Box::new(std::io::Error::other("Missing required argument: link name or path"))
            as Box<dyn std::error::Error>
    })?;

    // Add the link
    let (link_name, resolved_path) =
        add_workspace_link(&selected, name_or_path_val, path, &cwd)
            .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    if json {
        let output = WorkspaceMutationOutput {
            workspace: WorkspaceOutput {
                name: selected.name,
                root: selected
                    .root
                    .to_str()
                    .ok_or_else(|| {
                        Box::new(std::io::Error::other("Invalid UTF-8 in path"))
                            as Box<dyn std::error::Error>
                    })?
                    .to_string(),
                context: None,
                links: vec![],
            },
            link: WorkspaceLinkOutput {
                name: link_name,
                path: resolved_path,
            },
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Linked {} -> {}", link_name, resolved_path);
    }

    Ok(())
}

fn run_relink(
    name: &str,
    path: &str,
    workspace: Option<&str>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;

    // Resolve the workspace
    let selected = resolve_selected_workspace(workspace, &cwd, None)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    // Update the link
    let (link_name, resolved_path) = update_workspace_link(&selected, name, path, &cwd)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    if json {
        let output = WorkspaceMutationOutput {
            workspace: WorkspaceOutput {
                name: selected.name,
                root: selected
                    .root
                    .to_str()
                    .ok_or_else(|| {
                        Box::new(std::io::Error::other("Invalid UTF-8 in path"))
                            as Box<dyn std::error::Error>
                    })?
                    .to_string(),
                context: None,
                links: vec![],
            },
            link: WorkspaceLinkOutput {
                name: link_name,
                path: resolved_path,
            },
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Relinked {} -> {}", link_name, resolved_path);
    }

    Ok(())
}

fn run_doctor(workspace: Option<&str>, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;

    // Resolve the workspace
    let selected = resolve_selected_workspace(workspace, &cwd, None)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    let root_str = selected
        .root
        .to_str()
        .ok_or_else(|| Box::new(std::io::Error::other("Invalid UTF-8 in path")))?
        .to_string();

    let planning_path = selected.root.join("changes");
    let planning_path_str = planning_path
        .to_str()
        .ok_or_else(|| Box::new(std::io::Error::other("Invalid UTF-8 in path")))?
        .to_string();

    let view_state = read_workspace_view_state(&selected.root)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    let state_path = selected.root.join(".openspec-workspace/view.yaml");
    let state_path_str = state_path
        .to_str()
        .ok_or_else(|| Box::new(std::io::Error::other("Invalid UTF-8 in path")))?
        .to_string();

    // Get context string if present
    let context = if view_state.context.is_some() {
        Some("initiative".to_string())
    } else {
        None
    };

    // Check links and their existence
    let mut links = Vec::new();
    for (link_name, link_path) in &view_state.links {
        let exists = if let Some(path) = link_path {
            Path::new(path).exists()
        } else {
            false
        };

        links.push(WorkspaceDoctorLink {
            name: link_name.clone(),
            path: link_path.as_ref().map(|p| p.to_string()),
            exists,
        });
    }
    links.sort_by(|a, b| a.name.cmp(&b.name));

    let mut status = Vec::new();
    if has_workspace_skill_profile_drift(view_state.workspace_skills.as_ref()) {
        status.push(WorkspaceStatus::warning(
            "workspace_skills_out_of_sync",
            "Workspace-local agent skills are out of sync with the active global profile.",
        ));
    }

    if json {
        let output = WorkspaceDoctorOutput {
            workspace: WorkspaceDoctorWorkspaceInfo {
                name: selected.name,
                root: root_str,
                state_path: state_path_str,
                planning_path: planning_path_str,
                context,
                links,
            },
            status,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("\n{}", selected.name);
        println!("  root: {}", root_str);
        println!("  state: {}", state_path_str);
        println!("  planning: {}", planning_path_str);
        if let Some(ctx) = context {
            println!("  context: {}", ctx);
        }
        println!("\n  links:");
        if links.is_empty() {
            println!("    (none)");
        } else {
            for link in &links {
                let status_str = if link.exists { "ok" } else { "missing" };
                if let Some(path) = &link.path {
                    println!("    {} -> {} [{}]", link.name, path, status_str);
                } else {
                    println!("    {} -> (no path) [missing]", link.name);
                }
            }
        }
        for s in &status {
            println!("\n  [{}] {}: {}", s.level, s.code, s.message);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::workspace::infer_link_name;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_selected_workspace_from_name() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let ws_root = tmpdir.path();

        // Create workspace structure
        let metadata_dir = ws_root.join(".openspec-workspace");
        std::fs::create_dir_all(&metadata_dir).expect("failed to create metadata dir");

        let view_yaml = r#"version: 1
name: test-ws
context: null
links: {}
"#;
        std::fs::write(metadata_dir.join("view.yaml"), view_yaml)
            .expect("failed to write view.yaml");

        // Create and save a registry entry
        let mut registry = crate::core::workspace::WorkspaceRegistryState {
            version: 1,
            workspaces: std::collections::BTreeMap::new(),
        };
        let ws_root_str = ws_root.to_str().unwrap().to_string();
        registry
            .workspaces
            .insert("test-ws".to_string(), ws_root_str.clone());

        crate::core::workspace::save_workspace_registry(&registry, Some(tmpdir.path()))
            .expect("failed to save registry");

        // Now resolve the workspace
        let result = resolve_selected_workspace(Some("test-ws"), tmpdir.path(), Some(tmpdir.path()));
        assert!(result.is_ok());
        let selected = result.unwrap();
        assert_eq!(selected.name, "test-ws");
        assert_eq!(selected.root, ws_root);
    }

    #[test]
    fn test_resolve_selected_workspace_from_cwd() {
        let tmpdir = TempDir::new().expect("failed to create tempdir");
        let ws_root = tmpdir.path();

        // Create workspace structure
        let metadata_dir = ws_root.join(".openspec-workspace");
        std::fs::create_dir_all(&metadata_dir).expect("failed to create metadata dir");

        let view_yaml = r#"version: 1
name: test-ws
context: null
links: {}
"#;
        std::fs::write(metadata_dir.join("view.yaml"), view_yaml)
            .expect("failed to write view.yaml");

        // Resolve from cwd inside the workspace
        let result = resolve_selected_workspace(None, ws_root, None);
        assert!(result.is_ok());
        let selected = result.unwrap();
        assert_eq!(selected.name, "test-ws");
    }

    #[test]
    fn test_infer_link_name_from_path() {
        let path = std::path::Path::new("/home/user/my-repo");
        assert_eq!(infer_link_name(path), "my-repo");
    }
}
