use crate::cli::args::WorkspaceCommands;
use crate::core::workspace::{
    add_workspace_link, get_workspace_opener_executable, get_workspace_opener_label,
    has_workspace_skill_profile_drift, is_workspace_executable_available, is_workspace_root,
    list_workspace_registry_entries, load_workspace_registry, parse_workspace_preferred_opener_value,
    read_workspace_view_state, resolve_selected_workspace, sync_workspace_open_surface,
    update_workspace_link, OpenerKind, PreferredOpener, WorkspaceStatus,
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
        WorkspaceCommands::Setup {
            name,
            links,
            opener,
            tools,
            json,
        } => run_setup(name, links, opener, tools, json),
        WorkspaceCommands::Update {
            name,
            workspace,
            tools,
            json,
        } => run_update(name, workspace, tools, json),
        WorkspaceCommands::Open {
            name,
            workspace,
            agent,
            editor,
            json,
        } => run_open(name, workspace, agent, editor, json),
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

fn run_setup(
    name: Option<String>,
    links_input: Vec<String>,
    opener: Option<String>,
    tools: Option<String>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;

    // Validate workspace name was provided
    let workspace_name = name.ok_or_else(|| {
        Box::new(std::io::Error::other("Missing required argument --name"))
            as Box<dyn std::error::Error>
    })?;

    // Validate at least one link was provided
    if links_input.is_empty() {
        return Err(Box::new(std::io::Error::other(
            "workspace setup requires --name <name> and at least one --link <path>."
        )));
    }

    // Parse setup links
    let links = crate::core::workspace::parse_setup_links(&links_input, &cwd)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    // Parse the preferred opener if provided
    let preferred_opener = if let Some(opener_str) = opener {
        Some(
            crate::core::workspace::parse_workspace_preferred_opener_value(&opener_str)
                .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?,
        )
    } else {
        None
    };

    // Parse the tools list if provided
    let tools_list = tools.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());

    // Create the managed workspace
    let result = crate::core::workspace::create_managed_workspace(
        &workspace_name,
        links,
        preferred_opener,
        tools_list,
        None,
    )
    .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    if json {
        #[derive(serde::Serialize)]
        struct SetupOutput {
            workspace: String,
            root: String,
        }

        let output = SetupOutput {
            workspace: result.name,
            root: result
                .root
                .to_str()
                .ok_or_else(|| {
                    Box::new(std::io::Error::other("Invalid UTF-8 in workspace path"))
                        as Box<dyn std::error::Error>
                })?
                .to_string(),
        };

        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Workspace '{}' created at {}", result.name, result.root.display());
    }

    Ok(())
}

fn run_update(
    _name: Option<String>,
    workspace: Option<String>,
    tools: Option<String>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;

    // Resolve the workspace
    let selected = resolve_selected_workspace(workspace.as_deref(), &cwd, None)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    // Read the current view state
    let mut view_state = read_workspace_view_state(&selected.root)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    // Update tools if provided
    if let Some(tools_str) = tools {
        let tools_list: Vec<String> = tools_str.split(',').map(|s| s.trim().to_string()).collect();
        view_state.tools = Some(tools_list);
    }

    // Write the updated view state
    crate::core::workspace::write_workspace_view_state(&selected.root, &view_state)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    // Sync the open surface
    crate::core::workspace::sync_workspace_open_surface(&selected.root, &view_state, None)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    if json {
        #[derive(serde::Serialize)]
        struct UpdateOutput {
            workspace: String,
        }

        let output = UpdateOutput {
            workspace: selected.name.clone(),
        };

        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Updated workspace '{}'", selected.name);
    }

    Ok(())
}

fn run_open(
    name: Option<String>,
    workspace: Option<String>,
    agent: Option<String>,
    editor: bool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;

    // --agent and --editor are mutually exclusive opener selectors.
    if editor && agent.is_some() {
        return Err("Pass only one of --agent <tool> or --editor.".into());
    }

    // Allow either the positional name or --workspace (must agree if both given).
    let selector = match (name.as_deref(), workspace.as_deref()) {
        (Some(a), Some(b)) if a != b => {
            return Err(format!(
                "Conflicting workspace selectors: positional '{a}' and --workspace '{b}'."
            )
            .into());
        }
        (Some(a), _) => Some(a),
        (None, b) => b,
    };

    let selected = resolve_selected_workspace(selector, &cwd, None)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    let view_state = read_workspace_view_state(&selected.root)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    // Refresh the open surface (AGENTS.md + <name>.code-workspace) before launching.
    sync_workspace_open_surface(&selected.root, &view_state, None)
        .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;

    // Resolve the opener: explicit --editor / --agent wins, then the stored preferred opener,
    // else default to the VS Code editor.
    let opener: PreferredOpener = if editor {
        PreferredOpener {
            kind: OpenerKind::Editor,
            id: "vscode".to_string(),
        }
    } else if let Some(agent_id) = agent.as_deref() {
        let parsed = parse_workspace_preferred_opener_value(agent_id)
            .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)?;
        if parsed.kind != OpenerKind::Agent {
            return Err(format!("'{agent_id}' is not an agent opener. Use --editor for VS Code.").into());
        }
        parsed
    } else if let Some(stored) = view_state.preferred_opener.clone() {
        stored
    } else {
        PreferredOpener {
            kind: OpenerKind::Editor,
            id: "vscode".to_string(),
        }
    };

    let executable = get_workspace_opener_executable(&opener);
    let label = get_workspace_opener_label(&opener);

    if !is_workspace_executable_available(&executable) {
        return Err(format!(
            "Opener '{label}' is not available ({executable} not found on PATH)."
        )
        .into());
    }

    let workspace_root_str = selected.root.to_string_lossy().to_string();

    // Launch the opener against the workspace root, detached (do not wait for exit).
    std::process::Command::new(&executable)
        .arg(&selected.root)
        .spawn()
        .map_err(|e| {
            Box::new(std::io::Error::other(format!(
                "Failed to launch {executable}: {e}"
            ))) as Box<dyn std::error::Error>
        })?;

    if json {
        #[derive(serde::Serialize)]
        struct OpenerOutput {
            label: String,
            executable: String,
        }
        #[derive(serde::Serialize)]
        struct LaunchOutput {
            attempted: bool,
        }
        #[derive(serde::Serialize)]
        struct WorkspaceRef {
            name: String,
            root: String,
        }
        #[derive(serde::Serialize)]
        struct OpenOutput {
            workspace: WorkspaceRef,
            opener: OpenerOutput,
            launch: LaunchOutput,
        }

        let output = OpenOutput {
            workspace: WorkspaceRef {
                name: selected.name.clone(),
                root: workspace_root_str,
            },
            opener: OpenerOutput { label, executable },
            launch: LaunchOutput { attempted: true },
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Opening '{}' with {}...", selected.name, label);
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
