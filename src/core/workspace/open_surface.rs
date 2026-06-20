use crate::core::workspace::foundation::{
    get_workspace_code_workspace_file_name, get_workspace_code_workspace_path,
    get_workspace_context_initiative_id, ContextStoreSelector, WorkspaceContext, WorkspaceViewState,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

// Constants (must match upstream exactly)
pub const WORKSPACE_GUIDANCE_START_MARKER: &str = "<!-- OPENSPEC:WORKSPACE-GUIDANCE:START -->";
pub const WORKSPACE_GUIDANCE_END_MARKER: &str = "<!-- OPENSPEC:WORKSPACE-GUIDANCE:END -->";
pub const WORKSPACE_OPEN_ROOT_FOLDER_LABEL: &str = "OpenSpec workspace";
pub const WORKSPACE_OPEN_INITIATIVE_FOLDER_LABEL: &str = "Initiative context";

pub const WORKSPACE_GUIDANCE_BODY: &str = r#"# OpenSpec Workspace Guidance

This directory is an OpenSpec workspace: a local working view over context stores, initiatives, repos, and folders.

- Use this workspace to open the local view of coordinated work.
- Use initiatives for durable cross-team or cross-repo intent, decisions, requirements, and coordination context.
- Use repo-local OpenSpec changes for implementation plans owned by a repo or team.
- Use linked repos and folders to inspect context, understand ownership, and make edits in the place that owns the work.
- Keep workspace-local files focused on local paths, opener state, agent setup, and other machine-specific view state.
- Use OpenSpec workspace commands instead of hand-editing `.openspec-workspace/view.yaml`.
- If this workspace contains legacy or beta workspace-level planning files, treat them as compatibility context unless the user explicitly asks to use that beta flow."#;

// Types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedContextStoreRef {
    pub id: String,
    pub root: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedInitiativeRef {
    pub id: String,
    pub title: String,
    pub root: String,
    pub metadata_path: String,
    pub store_path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceOpenResolvedContext {
    pub context_store: ResolvedContextStoreRef,
    pub initiative: ResolvedInitiativeRef,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceOpenLink {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceSkippedReason {
    MissingLocalPath,
    PathMissing,
}

impl std::fmt::Display for WorkspaceSkippedReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceSkippedReason::MissingLocalPath => write!(f, "missing-local-path"),
            WorkspaceSkippedReason::PathMissing => write!(f, "path-missing"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceSkippedOpenLink {
    pub name: String,
    pub path: Option<String>,
    pub reason: WorkspaceSkippedReason,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceOpenSurfaceLinks {
    pub links: Vec<WorkspaceOpenLink>,
    pub skipped: Vec<WorkspaceSkippedOpenLink>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceOpenSurfaceGeneration {
    pub agents_path: String,
    pub code_workspace_path: String,
}

// Helper to format guidance path list
fn format_guidance_path_list(
    items: &[(&str, &str)],
) -> String {
    if items.is_empty() {
        return "- None selected yet.".to_string();
    }

    items
        .iter()
        .map(|(label, path)| format!("- {}: {}", label, path))
        .collect::<Vec<_>>()
        .join("\n")
}

// Build the context guidance section
fn build_workspace_context_guidance(
    view_state: &WorkspaceViewState,
    resolved_context: Option<&WorkspaceOpenResolvedContext>,
) -> String {
    let linked_roots: Vec<(&str, &str)> = view_state
        .links
        .iter()
        .filter_map(|(name, path)| path.as_ref().map(|p| (name.as_str(), p.as_str())))
        .collect();

    if view_state.context.is_none() {
        return format!(
            "## Local View

This workspace is not bound to an initiative. It is still a first-class local view over selected repos or folders.

## Linked Implementation Context

{}",
            format_guidance_path_list(&linked_roots)
        );
    }

    let context = view_state.context.as_ref().unwrap();
    let (store_id, selector) = match context {
        WorkspaceContext::Initiative { store, .. } => (&store.id, &store.selector),
    };

    let stored_context_store = match selector {
        ContextStoreSelector::Path { path, .. } => format!("{} via {}", store_id, path),
        ContextStoreSelector::Registry { .. } => store_id.clone(),
    };

    let stored_initiative_id = get_workspace_context_initiative_id(context);

    let context_lines = if let Some(resolved) = resolved_context {
        format!(
            "- Context store: {} ({})\n\
             - Initiative: {} ({})\n\
             - Initiative title: {}\n\
             - Initiative metadata: {}\n\
             - Broader context may exist in the context store, but this workspace opens the selected initiative by default.",
            resolved.context_store.id,
            resolved.context_store.root,
            resolved.initiative.id,
            resolved.initiative.root,
            resolved.initiative.title,
            resolved.initiative.metadata_path
        )
    } else {
        format!(
            "- Context store: {}\n\
             - Initiative: {}\n\
             - Run `openspec workspace open --json` to refresh resolved local paths for this view.",
            stored_context_store,
            stored_initiative_id
        )
    };

    format!(
        "## Selected Initiative Context

{}

## Advisory Edit Boundaries

- Treat initiative and context-store files as shared coordination context.
- Treat linked repos and folders as local implementation context when the user has selected them.
- These boundaries are advisory in this OpenSpec version; use judgment and repo ownership when editing.

## Linked Implementation Context

{}",
        context_lines,
        format_guidance_path_list(&linked_roots)
    )
}

/// Build the workspace guidance block with markers and optional context guidance.
pub fn build_workspace_guidance_block(
    view_state: Option<&WorkspaceViewState>,
    resolved_context: Option<&WorkspaceOpenResolvedContext>,
) -> String {
    let context_guidance = if let Some(vs) = view_state {
        format!(
            "\n\n{}",
            build_workspace_context_guidance(vs, resolved_context)
        )
    } else {
        String::new()
    };

    format!(
        "{}\n{}{}{}",
        WORKSPACE_GUIDANCE_START_MARKER, WORKSPACE_GUIDANCE_BODY, context_guidance, WORKSPACE_GUIDANCE_END_MARKER
    )
}

/// Apply workspace guidance block to existing content, replacing or appending as needed.
pub fn apply_workspace_guidance_block(
    existing: &str,
    view_state: Option<&WorkspaceViewState>,
    resolved_context: Option<&WorkspaceOpenResolvedContext>,
) -> Result<String, String> {
    let block = build_workspace_guidance_block(view_state, resolved_context);
    let start_index = existing.find(WORKSPACE_GUIDANCE_START_MARKER);
    let end_index = existing.find(WORKSPACE_GUIDANCE_END_MARKER);

    if start_index.is_some() || end_index.is_some() {
        match (start_index, end_index) {
            (Some(start), Some(end)) if end > start => {
                let before = existing[..start].trim_end();
                let after = existing[end + WORKSPACE_GUIDANCE_END_MARKER.len()..].trim_start();
                let prefix = if !before.is_empty() {
                    format!("{}\n\n", before)
                } else {
                    String::new()
                };
                let suffix = if !after.is_empty() {
                    format!("\n\n{}\n", after.trim_end())
                } else {
                    "\n".to_string()
                };
                Ok(format!("{}{}{}", prefix, block, suffix))
            }
            _ => Err("Invalid OpenSpec workspace guidance marker state in AGENTS.md.".to_string()),
        }
    } else if existing.trim().is_empty() {
        Ok(format!("{}\n", block))
    } else {
        Ok(format!("{}\n\n{}\n", existing.trim_end(), block))
    }
}

/// Build the code-workspace JSON content.
pub fn build_workspace_code_workspace_content(
    links: &[WorkspaceOpenLink],
    resolved_context: Option<&WorkspaceOpenResolvedContext>,
) -> String {
    let mut folders: Vec<serde_json::Map<String, serde_json::Value>> =
        links
            .iter()
            .map(|link| {
                let mut folder = serde_json::Map::new();
                folder.insert("name".to_string(), serde_json::Value::String(link.name.clone()));
                folder.insert("path".to_string(), serde_json::Value::String(link.path.clone()));
                folder
            })
            .collect();

    if let Some(resolved) = resolved_context {
        let mut initiative_folder = serde_json::Map::new();
        initiative_folder.insert(
            "name".to_string(),
            serde_json::Value::String(WORKSPACE_OPEN_INITIATIVE_FOLDER_LABEL.to_string()),
        );
        initiative_folder.insert(
            "path".to_string(),
            serde_json::Value::String(resolved.initiative.root.clone()),
        );
        folders.push(initiative_folder);
    }

    let mut root_folder = serde_json::Map::new();
    root_folder.insert(
        "name".to_string(),
        serde_json::Value::String(WORKSPACE_OPEN_ROOT_FOLDER_LABEL.to_string()),
    );
    root_folder.insert("path".to_string(), serde_json::Value::String(".".to_string()));
    folders.push(root_folder);

    let mut root = serde_json::Map::new();
    root.insert(
        "folders".to_string(),
        serde_json::Value::Array(folders.into_iter().map(serde_json::Value::Object).collect()),
    );

    let content = serde_json::to_string_pretty(&root).unwrap_or_else(|_| "{}".to_string());
    format!("{}\n", content)
}

/// Resolve workspace open links, checking which ones are valid.
pub fn resolve_workspace_open_links(
    view_state: &WorkspaceViewState,
) -> WorkspaceOpenSurfaceLinks {
    let mut links = Vec::new();
    let mut skipped = Vec::new();

    let mut link_names: Vec<_> = view_state.links.keys().collect();
    link_names.sort();

    for link_name in link_names {
        let local_path = view_state.links[link_name].as_ref();

        match local_path {
            None => {
                skipped.push(WorkspaceSkippedOpenLink {
                    name: link_name.clone(),
                    path: None,
                    reason: WorkspaceSkippedReason::MissingLocalPath,
                });
            }
            Some(path) => {
                if Path::new(path).is_dir() {
                    links.push(WorkspaceOpenLink {
                        name: link_name.clone(),
                        path: path.clone(),
                    });
                } else {
                    skipped.push(WorkspaceSkippedOpenLink {
                        name: link_name.clone(),
                        path: Some(path.clone()),
                        reason: WorkspaceSkippedReason::PathMissing,
                    });
                }
            }
        }
    }

    WorkspaceOpenSurfaceLinks { links, skipped }
}

/// Sync the workspace open surface: write AGENTS.md and .code-workspace file.
pub fn sync_workspace_open_surface(
    workspace_root: &Path,
    view_state: &WorkspaceViewState,
    resolved_context: Option<&WorkspaceOpenResolvedContext>,
) -> Result<(WorkspaceOpenSurfaceLinks, WorkspaceOpenSurfaceGeneration), String> {
    use crate::core::workspace::foundation::write_file_atomically;

    // Resolve links
    let open_links = resolve_workspace_open_links(view_state);

    // Write AGENTS.md
    let agents_path = workspace_root.join("AGENTS.md");
    let existing_content = std::fs::read_to_string(&agents_path).unwrap_or_default();
    let new_content = apply_workspace_guidance_block(&existing_content, Some(view_state), resolved_context)?;
    write_file_atomically(&agents_path, &new_content)
        .map_err(|e| format!("Failed to write AGENTS.md: {}", e))?;

    // Write .code-workspace file
    let code_workspace_path = get_workspace_code_workspace_path(workspace_root, &view_state.name)?;
    let workspace_content = build_workspace_code_workspace_content(&open_links.links, resolved_context);
    write_file_atomically(&code_workspace_path, &workspace_content)
        .map_err(|e| format!("Failed to write code-workspace file: {}", e))?;

    // Cleanup legacy .gitignore
    let gitignore_path = workspace_root.join(".gitignore");
    if let Ok(gitignore_content) = std::fs::read_to_string(&gitignore_path) {
        let file_name = get_workspace_code_workspace_file_name(&view_state.name)?;
        let non_empty_lines: Vec<_> = gitignore_content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();

        if non_empty_lines.len() == 1 && non_empty_lines[0].trim() == file_name {
            let _ = std::fs::remove_file(&gitignore_path);
        }
    }

    Ok((
        open_links,
        WorkspaceOpenSurfaceGeneration {
            agents_path: agents_path.to_string_lossy().to_string(),
            code_workspace_path: code_workspace_path.to_string_lossy().to_string(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::workspace::foundation::{ContextStoreBinding, ContextStoreSelector, WorkspaceInitiativeRef};
    use std::collections::BTreeMap;
    use tempfile::TempDir;

    fn create_test_view_state(name: &str, with_context: bool) -> WorkspaceViewState {
        let mut links = BTreeMap::new();
        links.insert("repo".to_string(), Some("/path/to/repo".to_string()));
        links.insert("docs".to_string(), None);

        let context = if with_context {
            Some(WorkspaceContext::Initiative {
                store: ContextStoreBinding {
                    id: "store-1".to_string(),
                    selector: ContextStoreSelector::Registry {
                        id: "my-store".to_string(),
                    },
                },
                initiative: WorkspaceInitiativeRef {
                    id: "init-1".to_string(),
                },
            })
        } else {
            None
        };

        WorkspaceViewState {
            version: 1,
            name: name.to_string(),
            context,
            links,
            preferred_opener: None,
            tools: None,
            workspace_skills: None,
        }
    }

    #[test]
    fn test_guidance_block_contains_markers_and_body() {
        let block = build_workspace_guidance_block(None, None);
        assert!(block.contains(WORKSPACE_GUIDANCE_START_MARKER));
        assert!(block.contains(WORKSPACE_GUIDANCE_END_MARKER));
        assert!(block.contains("# OpenSpec Workspace Guidance"));
        assert!(block.contains("This directory is an OpenSpec workspace"));
    }

    #[test]
    fn test_guidance_block_with_view_state_includes_local_view() {
        let view_state = create_test_view_state("test-ws", false);
        let block = build_workspace_guidance_block(Some(&view_state), None);
        assert!(block.contains("## Local View"));
        assert!(block.contains("This workspace is not bound to an initiative"));
    }

    #[test]
    fn test_guidance_block_with_context_includes_initiative_section() {
        let view_state = create_test_view_state("test-ws", true);
        let block = build_workspace_guidance_block(Some(&view_state), None);
        assert!(block.contains("## Selected Initiative Context"));
        assert!(block.contains("## Advisory Edit Boundaries"));
    }

    #[test]
    fn test_apply_guidance_block_empty_existing() {
        let view_state = create_test_view_state("test-ws", false);
        let result = apply_workspace_guidance_block("", Some(&view_state), None);
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains(WORKSPACE_GUIDANCE_START_MARKER));
        assert!(content.ends_with("\n"));
    }

    #[test]
    fn test_apply_guidance_block_existing_without_markers() {
        let view_state = create_test_view_state("test-ws", false);
        let existing = "Some existing content";
        let result = apply_workspace_guidance_block(existing, Some(&view_state), None);
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("Some existing content"));
        assert!(content.contains(WORKSPACE_GUIDANCE_START_MARKER));
        // Should have content before the marker with blank lines
        assert!(content.contains("Some existing content\n\n"));
    }

    #[test]
    fn test_apply_guidance_block_existing_with_markers() {
        let view_state = create_test_view_state("test-ws", false);
        let existing = format!(
            "Before\n\n{}\nOld body\n{}\n\nAfter",
            WORKSPACE_GUIDANCE_START_MARKER, WORKSPACE_GUIDANCE_END_MARKER
        );
        let result = apply_workspace_guidance_block(&existing, Some(&view_state), None);
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("Before"));
        assert!(content.contains("After"));
        // Should not have duplicated markers
        assert_eq!(content.matches(WORKSPACE_GUIDANCE_START_MARKER).count(), 1);
        assert_eq!(content.matches(WORKSPACE_GUIDANCE_END_MARKER).count(), 1);
    }

    #[test]
    fn test_apply_guidance_block_only_start_marker_error() {
        let view_state = create_test_view_state("test-ws", false);
        let existing = format!("Content with {}", WORKSPACE_GUIDANCE_START_MARKER);
        let result = apply_workspace_guidance_block(&existing, Some(&view_state), None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Invalid OpenSpec workspace guidance marker state"));
    }

    #[test]
    fn test_code_workspace_content_has_folders() {
        let links = vec![
            WorkspaceOpenLink {
                name: "repo".to_string(),
                path: "/path/to/repo".to_string(),
            },
        ];
        let content = build_workspace_code_workspace_content(&links, None);
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.get("folders").is_some());
        let folders = parsed["folders"].as_array().unwrap();
        assert!(folders.len() > 0);
    }

    #[test]
    fn test_code_workspace_ends_with_root_folder() {
        let links = vec![
            WorkspaceOpenLink {
                name: "repo".to_string(),
                path: "/path/to/repo".to_string(),
            },
        ];
        let content = build_workspace_code_workspace_content(&links, None);
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        let folders = parsed["folders"].as_array().unwrap();
        let last_folder = &folders[folders.len() - 1];
        assert_eq!(
            last_folder.get("name").unwrap().as_str().unwrap(),
            WORKSPACE_OPEN_ROOT_FOLDER_LABEL
        );
        assert_eq!(last_folder.get("path").unwrap().as_str().unwrap(), ".");
    }

    #[test]
    fn test_code_workspace_includes_initiative_folder_when_context_given() {
        let links = vec![];
        let resolved_context = WorkspaceOpenResolvedContext {
            context_store: ResolvedContextStoreRef {
                id: "store-1".to_string(),
                root: "/path/to/store".to_string(),
            },
            initiative: ResolvedInitiativeRef {
                id: "init-1".to_string(),
                title: "Test Initiative".to_string(),
                root: "/path/to/initiative".to_string(),
                metadata_path: "/path/to/initiative/METADATA.md".to_string(),
                store_path: "/path/to/initiative/store.yaml".to_string(),
            },
        };
        let content = build_workspace_code_workspace_content(&links, Some(&resolved_context));
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        let folders = parsed["folders"].as_array().unwrap();
        assert_eq!(folders.len(), 2); // initiative + root
        assert_eq!(
            folders[0].get("name").unwrap().as_str().unwrap(),
            WORKSPACE_OPEN_INITIATIVE_FOLDER_LABEL
        );
    }

    #[test]
    fn test_resolve_links_missing_local_path() {
        let mut links = BTreeMap::new();
        links.insert("no-path".to_string(), None);

        let view_state = WorkspaceViewState {
            version: 1,
            name: "test-ws".to_string(),
            context: None,
            links,
            preferred_opener: None,
            tools: None,
            workspace_skills: None,
        };

        let result = resolve_workspace_open_links(&view_state);
        assert_eq!(result.links.len(), 0);
        assert_eq!(result.skipped.len(), 1);
        assert_eq!(result.skipped[0].reason, WorkspaceSkippedReason::MissingLocalPath);
        assert_eq!(result.skipped[0].path, None);
    }

    #[test]
    fn test_resolve_links_path_missing() {
        let mut links = BTreeMap::new();
        links.insert("bad-path".to_string(), Some("/nonexistent/path".to_string()));

        let view_state = WorkspaceViewState {
            version: 1,
            name: "test-ws".to_string(),
            context: None,
            links,
            preferred_opener: None,
            tools: None,
            workspace_skills: None,
        };

        let result = resolve_workspace_open_links(&view_state);
        assert_eq!(result.links.len(), 0);
        assert_eq!(result.skipped.len(), 1);
        assert_eq!(result.skipped[0].reason, WorkspaceSkippedReason::PathMissing);
        assert_eq!(
            result.skipped[0].path.as_ref().unwrap(),
            "/nonexistent/path"
        );
    }

    #[test]
    fn test_resolve_links_valid_directory() {
        let temp = TempDir::new().unwrap();
        let temp_path = temp.path().to_string_lossy().to_string();

        let mut links = BTreeMap::new();
        links.insert("valid".to_string(), Some(temp_path.clone()));

        let view_state = WorkspaceViewState {
            version: 1,
            name: "test-ws".to_string(),
            context: None,
            links,
            preferred_opener: None,
            tools: None,
            workspace_skills: None,
        };

        let result = resolve_workspace_open_links(&view_state);
        assert_eq!(result.links.len(), 1);
        assert_eq!(result.links[0].name, "valid");
        assert_eq!(result.links[0].path, temp_path);
        assert_eq!(result.skipped.len(), 0);
    }

    #[test]
    fn test_sync_workspace_open_surface_writes_files() {
        let temp = TempDir::new().unwrap();
        let workspace_root = temp.path();

        let mut links = BTreeMap::new();
        links.insert("test".to_string(), Some(workspace_root.to_string_lossy().to_string()));

        let view_state = WorkspaceViewState {
            version: 1,
            name: "my-ws".to_string(),
            context: None,
            links,
            preferred_opener: None,
            tools: None,
            workspace_skills: None,
        };

        let result = sync_workspace_open_surface(workspace_root, &view_state, None);
        assert!(result.is_ok());

        let (links_result, generation) = result.unwrap();
        assert!(!links_result.links.is_empty());

        // The returned generation paths point at the files that were written.
        let agents_path = workspace_root.join("AGENTS.md");
        let code_workspace_path = workspace_root.join("my-ws.code-workspace");
        assert_eq!(generation.agents_path, agents_path.to_string_lossy());
        assert_eq!(
            generation.code_workspace_path,
            code_workspace_path.to_string_lossy()
        );

        assert!(agents_path.exists());
        assert!(code_workspace_path.exists());

        let agents_content = std::fs::read_to_string(&agents_path).unwrap();
        assert!(agents_content.contains(WORKSPACE_GUIDANCE_START_MARKER));
    }

    #[test]
    fn test_sync_workspace_open_surface_no_duplication() {
        let temp = TempDir::new().unwrap();
        let workspace_root = temp.path();

        let mut links = BTreeMap::new();
        links.insert("test".to_string(), Some(workspace_root.to_string_lossy().to_string()));

        let view_state = WorkspaceViewState {
            version: 1,
            name: "my-ws".to_string(),
            context: None,
            links,
            preferred_opener: None,
            tools: None,
            workspace_skills: None,
        };

        // First sync
        let result1 = sync_workspace_open_surface(workspace_root, &view_state, None);
        assert!(result1.is_ok());

        // Second sync
        let result2 = sync_workspace_open_surface(workspace_root, &view_state, None);
        assert!(result2.is_ok());

        let agents_content = std::fs::read_to_string(workspace_root.join("AGENTS.md")).unwrap();
        // Should only have one set of markers
        assert_eq!(agents_content.matches(WORKSPACE_GUIDANCE_START_MARKER).count(), 1);
        assert_eq!(agents_content.matches(WORKSPACE_GUIDANCE_END_MARKER).count(), 1);
    }

    #[test]
    fn test_sync_workspace_cleans_up_legacy_gitignore() {
        let temp = TempDir::new().unwrap();
        let workspace_root = temp.path();

        // Create a .gitignore with the legacy pattern
        let gitignore_path = workspace_root.join(".gitignore");
        std::fs::write(&gitignore_path, "my-ws.code-workspace").unwrap();

        let mut links = BTreeMap::new();
        links.insert("test".to_string(), Some(workspace_root.to_string_lossy().to_string()));

        let view_state = WorkspaceViewState {
            version: 1,
            name: "my-ws".to_string(),
            context: None,
            links,
            preferred_opener: None,
            tools: None,
            workspace_skills: None,
        };

        sync_workspace_open_surface(workspace_root, &view_state, None).unwrap();

        // .gitignore should be removed
        assert!(!gitignore_path.exists());
    }

    #[test]
    fn test_sync_workspace_preserves_non_legacy_gitignore() {
        let temp = TempDir::new().unwrap();
        let workspace_root = temp.path();

        // Create a .gitignore with additional content
        let gitignore_path = workspace_root.join(".gitignore");
        std::fs::write(&gitignore_path, "my-ws.code-workspace\nother-pattern").unwrap();

        let mut links = BTreeMap::new();
        links.insert("test".to_string(), Some(workspace_root.to_string_lossy().to_string()));

        let view_state = WorkspaceViewState {
            version: 1,
            name: "my-ws".to_string(),
            context: None,
            links,
            preferred_opener: None,
            tools: None,
            workspace_skills: None,
        };

        sync_workspace_open_surface(workspace_root, &view_state, None).unwrap();

        // .gitignore should still exist
        assert!(gitignore_path.exists());
    }
}
