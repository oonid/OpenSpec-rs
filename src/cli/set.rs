use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::cli::args::SetCommands;
use crate::cli::new_change::validate_change_name;
use crate::core::collections::initiatives::{
    find_initiative_across_stores, read_initiative, resolve_selected_store,
};
use crate::core::schema::{ChangeMetadata, InitiativeLink};

pub fn run(cmd: SetCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        SetCommands::Change {
            name,
            initiative,
            store,
            store_path,
            json,
        } => run_set_change(
            name.as_deref(),
            initiative.as_deref(),
            store.as_deref(),
            store_path.as_deref(),
            json,
        ),
    }
}

/// Outcome of comparing an existing change link against the requested one.
#[derive(Debug, Clone, PartialEq)]
pub enum LinkDecision {
    /// Already linked to the same initiative; idempotent no-op.
    NoOp,
    /// Not currently linked; write the new link.
    Write,
    /// Already linked to a different initiative; reject.
    Conflict,
}

/// Decide what to do given the existing change link and the requested target link.
pub fn decide_link(existing: Option<&InitiativeLink>, target: &InitiativeLink) -> LinkDecision {
    match existing {
        Some(current) if current == target => LinkDecision::NoOp,
        Some(_) => LinkDecision::Conflict,
        None => LinkDecision::Write,
    }
}

fn format_link(link: &InitiativeLink) -> String {
    format!("{}/{}", link.store, link.id)
}

#[derive(Serialize)]
struct ChangeInfo {
    id: String,
    path: String,
    #[serde(rename = "metadataPath")]
    metadata_path: String,
    schema: String,
}

#[derive(Serialize)]
struct SetChangeOutput {
    change: ChangeInfo,
    initiative: InitiativeLink,
    linked: bool,
}

#[derive(Serialize)]
struct SetChangeErrorOutput {
    change: Option<()>,
    error: String,
}

fn run_set_change(
    name: Option<&str>,
    initiative: Option<&str>,
    store: Option<&str>,
    store_path: Option<&str>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match set_change_inner(name, initiative, store, store_path) {
        Ok((change_id, change_dir, metadata, linked)) => {
            let link = metadata
                .initiative
                .clone()
                .expect("link present on success");
            if json {
                let output = SetChangeOutput {
                    change: ChangeInfo {
                        id: change_id.clone(),
                        path: change_dir.to_string_lossy().to_string(),
                        metadata_path: change_dir
                            .join(".openspec.yaml")
                            .to_string_lossy()
                            .to_string(),
                        schema: metadata.schema.clone(),
                    },
                    initiative: link,
                    linked,
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                let verb = if linked {
                    "Linked"
                } else {
                    "Change already linked"
                };
                println!("{}: {}", verb, change_id);
                println!("Initiative: {}", format_link(&link));
                println!("Metadata: {}", change_dir.join(".openspec.yaml").display());
            }
            Ok(())
        }
        Err(message) => {
            if json {
                let output = SetChangeErrorOutput {
                    change: None,
                    error: message,
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
                std::process::exit(1);
            }
            Err(message.into())
        }
    }
}

/// Core logic, returning `(change_id, change_dir, metadata_with_link, linked)` on success
/// or a human-readable error message string.
fn set_change_inner(
    name: Option<&str>,
    initiative: Option<&str>,
    store: Option<&str>,
    store_path: Option<&str>,
) -> Result<(String, PathBuf, ChangeMetadata, bool), String> {
    let name = name.ok_or_else(|| "Missing required argument <name>".to_string())?;

    let initiative_id = match initiative {
        Some(id) if !id.trim().is_empty() => id,
        _ => {
            return Err("Pass --initiative <id> to set a change initiative link.".to_string());
        }
    };

    // Validate the change name to prevent path traversal.
    let validation = validate_change_name(name);
    if !validation.valid {
        return Err(format!(
            "Invalid change name '{}': {}",
            name,
            validation
                .error
                .unwrap_or_else(|| "invalid name".to_string())
        ));
    }

    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
    let change_dir = project_root.join("openspec").join("changes").join(name);

    if !change_dir.is_dir() {
        return Err(format!("Change '{}' not found.", name));
    }

    // Resolve the initiative link.
    let link = resolve_initiative_link(initiative_id, store, store_path)?;

    // Read existing metadata (if any) and decide what to do.
    let metadata_path = change_dir.join(".openspec.yaml");
    let existing_metadata = read_change_metadata(&metadata_path)?;
    let base_metadata = existing_metadata.unwrap_or_else(|| ChangeMetadata {
        schema: crate::cli::new_change::DEFAULT_SCHEMA.to_string(),
        created: None,
        goal: None,
        affected_areas: None,
        initiative: None,
    });

    match decide_link(base_metadata.initiative.as_ref(), &link) {
        LinkDecision::NoOp => {
            // Idempotent: already linked to the same initiative.
            Ok((name.to_string(), change_dir, base_metadata, false))
        }
        LinkDecision::Conflict => {
            let current = base_metadata.initiative.as_ref().unwrap();
            Err(format!(
                "Change '{}' is already linked to initiative {}.",
                name,
                format_link(current)
            ))
        }
        LinkDecision::Write => {
            let updated = ChangeMetadata {
                schema: base_metadata.schema.clone(),
                created: base_metadata.created.clone(),
                goal: base_metadata.goal.clone(),
                affected_areas: base_metadata.affected_areas.clone(),
                initiative: Some(link),
            };
            write_change_metadata(&metadata_path, &updated)?;
            Ok((name.to_string(), change_dir, updated, true))
        }
    }
}

/// Resolve the requested initiative into a concrete `{ store, id }` link.
pub(crate) fn resolve_initiative_link(
    initiative_id: &str,
    store: Option<&str>,
    store_path: Option<&str>,
) -> Result<InitiativeLink, String> {
    if store.is_some() || store_path.is_some() {
        let selected = resolve_selected_store(store, store_path, None)?;
        let found = read_initiative(&selected.root, initiative_id)?;
        if found.is_none() {
            return Err(format!(
                "Initiative '{}' was not found in context store '{}'.",
                initiative_id, selected.id
            ));
        }
        return Ok(InitiativeLink {
            store: selected.id,
            id: initiative_id.to_string(),
        });
    }

    let matches = find_initiative_across_stores(initiative_id, None)?;
    match matches.len() {
        0 => Err(format!(
            "Initiative '{}' was not found in registered context stores.",
            initiative_id
        )),
        1 => {
            let (selected, _) = matches.into_iter().next().unwrap();
            Ok(InitiativeLink {
                store: selected.id,
                id: initiative_id.to_string(),
            })
        }
        _ => Err(format!(
            "Initiative '{}' exists in multiple context stores. Use --store <store>.",
            initiative_id
        )),
    }
}

fn read_change_metadata(path: &Path) -> Result<Option<ChangeMetadata>, String> {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let metadata: ChangeMetadata = serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            Ok(Some(metadata))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Failed to read {}: {}", path.display(), e)),
    }
}

fn write_change_metadata(path: &Path, metadata: &ChangeMetadata) -> Result<(), String> {
    let content = serde_yaml::to_string(metadata)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    std::fs::write(path, content).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link(store: &str, id: &str) -> InitiativeLink {
        InitiativeLink {
            store: store.to_string(),
            id: id.to_string(),
        }
    }

    #[test]
    fn test_decide_link_write_when_no_existing() {
        let target = link("team", "roadmap");
        assert_eq!(decide_link(None, &target), LinkDecision::Write);
    }

    #[test]
    fn test_decide_link_noop_when_same() {
        let existing = link("team", "roadmap");
        let target = link("team", "roadmap");
        assert_eq!(decide_link(Some(&existing), &target), LinkDecision::NoOp);
    }

    #[test]
    fn test_decide_link_conflict_on_different_id() {
        let existing = link("team", "roadmap");
        let target = link("team", "other");
        assert_eq!(
            decide_link(Some(&existing), &target),
            LinkDecision::Conflict
        );
    }

    #[test]
    fn test_decide_link_conflict_on_different_store() {
        let existing = link("team", "roadmap");
        let target = link("other", "roadmap");
        assert_eq!(
            decide_link(Some(&existing), &target),
            LinkDecision::Conflict
        );
    }

    #[test]
    fn test_read_write_change_metadata_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".openspec.yaml");

        // Missing file => None.
        assert!(read_change_metadata(&path).unwrap().is_none());

        let meta = ChangeMetadata {
            schema: "spec-driven".to_string(),
            created: Some("2026-01-01".to_string()),
            goal: None,
            affected_areas: None,
            initiative: Some(link("team", "roadmap")),
        };
        write_change_metadata(&path, &meta).unwrap();

        let read_back = read_change_metadata(&path).unwrap().unwrap();
        assert_eq!(read_back, meta);
    }

    #[test]
    fn test_read_legacy_schema_only_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".openspec.yaml");
        std::fs::write(&path, "schema: spec-driven\ncreated: 2026-01-01\n").unwrap();

        let meta = read_change_metadata(&path).unwrap().unwrap();
        assert_eq!(meta.schema, "spec-driven");
        assert_eq!(meta.created.as_deref(), Some("2026-01-01"));
        assert!(meta.initiative.is_none());
    }
}
