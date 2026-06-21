use std::path::Path;

use crate::cli::args::SchemaCommands;
use crate::core::error::{OpenSpecError, Result};
use crate::core::schema::{
    get_embedded_spec_driven_schema, get_package_schemas_dir, get_project_schemas_dir,
    get_user_schemas_dir, list_schemas, load_schema, resolve_schema, Artifact, SchemaSource,
    SchemaYaml,
};

const DEFAULT_SCHEMA: &str = "spec-driven";

fn source_label(source: &SchemaSource) -> &'static str {
    match source {
        SchemaSource::Project => "project",
        SchemaSource::User => "user",
        SchemaSource::Package => "package",
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct ShadowJson {
    source: String,
    path: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ResolutionJson {
    name: String,
    source: String,
    path: String,
    shadows: Vec<ShadowJson>,
}

/// Inspect all three schema locations for `name`, returning the active source
/// (first that exists, in project > user > package precedence) followed by any
/// shadowed locations.
fn resolve_with_shadows(name: &str, project_root: &Path) -> Option<ResolutionJson> {
    let candidates = [
        (
            SchemaSource::Project,
            get_project_schemas_dir(project_root).join(name),
        ),
        (SchemaSource::User, get_user_schemas_dir().join(name)),
        (SchemaSource::Package, get_package_schemas_dir().join(name)),
    ];

    let mut existing: Vec<(SchemaSource, String)> = Vec::new();
    for (source, dir) in candidates {
        if dir.join("schema.yaml").exists() {
            existing.push((source, dir.display().to_string()));
        }
    }

    // Embedded fallback for spec-driven when no on-disk schema is present.
    if existing.is_empty() && name == DEFAULT_SCHEMA {
        return Some(ResolutionJson {
            name: name.to_string(),
            source: "package".to_string(),
            path: "embedded:spec-driven.yaml".to_string(),
            shadows: Vec::new(),
        });
    }

    if existing.is_empty() {
        return None;
    }

    let (active_source, active_path) = existing[0].clone();
    let shadows = existing[1..]
        .iter()
        .map(|(s, p)| ShadowJson {
            source: source_label(s).to_string(),
            path: p.clone(),
        })
        .collect();

    Some(ResolutionJson {
        name: name.to_string(),
        source: source_label(&active_source).to_string(),
        path: active_path,
        shadows,
    })
}

pub fn run(cmd: SchemaCommands) -> Result<()> {
    eprintln!("Note: Schema commands are experimental and may change.");
    match cmd {
        SchemaCommands::Which { name, all, json } => run_which(name.as_deref(), all, json),
        SchemaCommands::Validate { name, json } => run_validate(name.as_deref(), json),
        SchemaCommands::Fork {
            source,
            name,
            force,
            json,
        } => run_fork(&source, name.as_deref(), force, json),
        SchemaCommands::Init { name, force, json } => run_init(&name, force, json),
    }
}

/// Validate a schema name is kebab-case (lowercase alphanumerics with single
/// hyphen separators), matching the upstream `isValidSchemaName` check.
fn is_valid_schema_name(name: &str) -> bool {
    let re = regex::Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
    re.is_match(name)
}

/// The embedded template bodies, keyed by the template filename referenced from
/// the embedded spec-driven schema. A released binary has no `vendor/` directory,
/// so forking the embedded source materializes these into `templates/`.
fn embedded_template(filename: &str) -> Option<&'static str> {
    match filename {
        "proposal.md" => Some(include_str!("../embedded_templates/proposal.md")),
        "spec.md" => Some(include_str!("../embedded_templates/spec.md")),
        "design.md" => Some(include_str!("../embedded_templates/design.md")),
        "tasks.md" => Some(include_str!("../embedded_templates/tasks.md")),
        _ => None,
    }
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest).map_err(|e| OpenSpecError::IoWrite {
        path: dest.to_path_buf(),
        source: e,
    })?;
    let entries = std::fs::read_dir(src).map_err(|e| OpenSpecError::IoRead {
        path: src.to_path_buf(),
        source: e,
    })?;
    for entry in entries.flatten() {
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path).map_err(|e| OpenSpecError::IoWrite {
                path: dest_path.clone(),
                source: e,
            })?;
        }
    }
    Ok(())
}

fn write_file(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| OpenSpecError::IoWrite {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }
    std::fs::write(path, contents).map_err(|e| OpenSpecError::IoWrite {
        path: path.to_path_buf(),
        source: e,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
struct ForkJson {
    forked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct InitJson {
    created: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Emit a fork failure (JSON object or stderr lines) and return a nonzero error.
fn fork_error(error: String, extra_line: Option<&str>, json: bool) -> Result<()> {
    if json {
        let out = ForkJson {
            forked: false,
            name: None,
            path: None,
            error: Some(error.clone()),
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    }
    // main() prints the returned error once; fold the hint into the message and do not
    // eprintln here (avoids a duplicated "Error:" line).
    Err(OpenSpecError::Custom(compose_error(error, extra_line)))
}

fn init_error(error: String, extra_line: Option<&str>, json: bool) -> Result<()> {
    if json {
        let out = InitJson {
            created: false,
            name: None,
            path: None,
            error: Some(error.clone()),
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    }
    Err(OpenSpecError::Custom(compose_error(error, extra_line)))
}

fn compose_error(error: String, extra_line: Option<&str>) -> String {
    match extra_line {
        Some(line) => format!("{error}\n{line}"),
        None => error,
    }
}

fn run_fork(source: &str, name: Option<&str>, force: bool, json: bool) -> Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    let dest_name = match name {
        Some(n) => n.to_string(),
        None => format!("{}-custom", source),
    };

    if !is_valid_schema_name(&dest_name) {
        return fork_error(
            format!("Invalid schema name '{}'", dest_name),
            Some("Schema names must be kebab-case (e.g., my-workflow)"),
            json,
        );
    }

    // Resolve the source schema (project > user > package > embedded spec-driven).
    let resolved = match resolve_schema(source, Some(&project_root)) {
        Ok(r) => r,
        Err(OpenSpecError::SchemaNotFound { .. }) => {
            let available = list_schemas(Some(&project_root))
                .into_iter()
                .map(|s| s.name)
                .collect::<Vec<_>>()
                .join(", ");
            return fork_error(
                format!("Schema '{}' not found", source),
                Some(&format!("Available schemas: {}", available)),
                json,
            );
        }
        Err(e) => return fork_error(e.to_string(), None, json),
    };

    let dest_dir = get_project_schemas_dir(&project_root).join(&dest_name);

    if dest_dir.exists() {
        if !force {
            return fork_error(
                format!(
                    "Schema '{}' already exists at {}",
                    dest_name,
                    dest_dir.display()
                ),
                Some("Use --force to overwrite"),
                json,
            );
        }
        std::fs::remove_dir_all(&dest_dir).map_err(|e| OpenSpecError::IoWrite {
            path: dest_dir.clone(),
            source: e,
        })?;
    }

    // The resolved `path` is either an on-disk `.../schema.yaml` or the embedded
    // sentinel `embedded:spec-driven.yaml`. On-disk sources are copied verbatim;
    // the embedded source is materialized from baked-in content.
    let on_disk = !resolved.path.starts_with("embedded:");

    if on_disk {
        let source_schema_path = Path::new(&resolved.path);
        let source_dir = source_schema_path
            .parent()
            .ok_or_else(|| OpenSpecError::Custom("Source schema has no parent dir".to_string()))?;
        copy_dir_recursive(source_dir, &dest_dir)?;
        // Rewrite the name field in the copied schema.yaml.
        let dest_schema_path = dest_dir.join("schema.yaml");
        let mut schema = load_schema(&dest_schema_path)?;
        schema.name = dest_name.clone();
        write_file(&dest_schema_path, &serde_yaml::to_string(&schema)?)?;
    } else {
        // Materialize the embedded spec-driven schema + its templates.
        let mut schema = get_embedded_spec_driven_schema()?;
        schema.name = dest_name.clone();
        write_file(
            &dest_dir.join("schema.yaml"),
            &serde_yaml::to_string(&schema)?,
        )?;
        for artifact in &schema.artifacts {
            if let Some(body) = embedded_template(&artifact.template) {
                write_file(&dest_dir.join("templates").join(&artifact.template), body)?;
            }
        }
    }

    if json {
        let out = ForkJson {
            forked: true,
            name: Some(dest_name),
            path: Some(dest_dir.display().to_string()),
            error: None,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Forked '{}' to '{}'", source, dest_name);
        println!("  {}", dest_dir.display());
    }

    Ok(())
}

/// Default artifacts scaffolded by `schema init`, mirroring upstream's
/// `DEFAULT_ARTIFACTS`.
fn default_init_artifacts() -> Vec<Artifact> {
    vec![
        Artifact {
            id: "proposal".to_string(),
            generates: "proposal.md".to_string(),
            description: "High-level description of the change, its motivation, and scope"
                .to_string(),
            template: "proposal.md".to_string(),
            instruction: None,
            requires: vec![],
        },
        Artifact {
            id: "specs".to_string(),
            generates: "specs/**/*.md".to_string(),
            description: "Detailed specifications with requirements and scenarios".to_string(),
            template: "spec.md".to_string(),
            instruction: None,
            requires: vec!["proposal".to_string()],
        },
        Artifact {
            id: "design".to_string(),
            generates: "design.md".to_string(),
            description: "Technical design decisions and implementation approach".to_string(),
            template: "design.md".to_string(),
            instruction: None,
            requires: vec!["specs".to_string()],
        },
        Artifact {
            id: "tasks".to_string(),
            generates: "tasks.md".to_string(),
            description: "Implementation checklist with trackable tasks".to_string(),
            template: "tasks.md".to_string(),
            instruction: None,
            requires: vec!["design".to_string()],
        },
    ]
}

fn run_init(name: &str, force: bool, json: bool) -> Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    if !is_valid_schema_name(name) {
        return init_error(
            format!("Invalid schema name '{}'", name),
            Some("Schema names must be kebab-case (e.g., my-workflow)"),
            json,
        );
    }

    let schema_dir = get_project_schemas_dir(&project_root).join(name);

    if schema_dir.exists() {
        if !force {
            return init_error(
                format!(
                    "Schema '{}' already exists at {}",
                    name,
                    schema_dir.display()
                ),
                Some("Use --force to overwrite or \"openspec schema fork\" to copy"),
                json,
            );
        }
        std::fs::remove_dir_all(&schema_dir).map_err(|e| OpenSpecError::IoWrite {
            path: schema_dir.clone(),
            source: e,
        })?;
    }

    let artifacts = default_init_artifacts();
    let schema = SchemaYaml {
        name: name.to_string(),
        version: 1,
        description: Some(format!("Custom workflow schema for {}", name)),
        artifacts: artifacts.clone(),
        apply: Some(crate::core::schema::ApplyPhase {
            requires: vec!["tasks".to_string()],
            tracks: Some("tasks.md".to_string()),
            instruction: None,
        }),
    };

    write_file(
        &schema_dir.join("schema.yaml"),
        &serde_yaml::to_string(&schema)?,
    )?;
    for artifact in &artifacts {
        let body = embedded_template(&artifact.template).unwrap_or("");
        write_file(&schema_dir.join("templates").join(&artifact.template), body)?;
    }

    if json {
        let out = InitJson {
            created: true,
            name: Some(name.to_string()),
            path: Some(schema_dir.display().to_string()),
            error: None,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Created schema '{}'", name);
        println!("  {}", schema_dir.display());
    }

    Ok(())
}

fn run_which(name: Option<&str>, all: bool, json: bool) -> Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    if all {
        let schemas = list_schemas(Some(&project_root));
        let resolutions: Vec<ResolutionJson> = schemas
            .iter()
            .filter_map(|s| resolve_with_shadows(&s.name, &project_root))
            .collect();

        if json {
            println!("{}", serde_json::to_string_pretty(&resolutions)?);
        } else if resolutions.is_empty() {
            println!("No schemas found.");
        } else {
            print_grouped(&resolutions, "project", "Project schemas:");
            print_grouped(&resolutions, "user", "User schemas:");
            print_grouped(&resolutions, "package", "Package schemas:");
        }
        return Ok(());
    }

    let name = match name {
        Some(n) => n,
        None => {
            return Err(OpenSpecError::Custom(
                "Schema name is required (or use --all to list all schemas)".to_string(),
            ));
        }
    };

    let resolution = match resolve_with_shadows(name, &project_root) {
        Some(r) => r,
        None => {
            let available = list_schemas(Some(&project_root))
                .into_iter()
                .map(|s| s.name)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(OpenSpecError::schema_not_found(name, available));
        }
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&resolution)?);
    } else {
        println!("Schema: {}", resolution.name);
        println!("Source: {}", resolution.source);
        println!("Path: {}", resolution.path);
        if !resolution.shadows.is_empty() {
            println!();
            println!("Shadows:");
            for shadow in &resolution.shadows {
                println!("  {}: {}", shadow.source, shadow.path);
            }
        }
    }

    Ok(())
}

fn print_grouped(resolutions: &[ResolutionJson], source: &str, heading: &str) {
    let group: Vec<&ResolutionJson> = resolutions.iter().filter(|r| r.source == source).collect();
    if group.is_empty() {
        return;
    }
    println!();
    println!("{}", heading);
    for schema in group {
        if schema.shadows.is_empty() {
            println!("  {}", schema.name);
        } else {
            let sources: Vec<&str> = schema.shadows.iter().map(|s| s.source.as_str()).collect();
            println!("  {} (shadows: {})", schema.name, sources.join(", "));
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct ValidateJson {
    name: String,
    valid: bool,
    issues: Vec<String>,
}

fn run_validate(name: Option<&str>, json: bool) -> Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    let schema_name = name.unwrap_or(DEFAULT_SCHEMA);

    // Resolution surfaces structural/cycle errors as a load failure; treat those
    // as validation issues rather than propagating, so output stays consistent.
    let resolved = match resolve_schema(schema_name, Some(&project_root)) {
        Ok(r) => r,
        Err(OpenSpecError::SchemaNotFound { .. }) => {
            let available = list_schemas(Some(&project_root))
                .into_iter()
                .map(|s| s.name)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(OpenSpecError::schema_not_found(schema_name, available));
        }
        Err(e) => {
            let issues = vec![e.to_string()];
            return emit_validate(schema_name, false, issues, json);
        }
    };

    match resolved.schema.validate() {
        Ok(()) => emit_validate(schema_name, true, Vec::new(), json),
        Err(message) => emit_validate(schema_name, false, vec![message], json),
    }
}

fn emit_validate(name: &str, valid: bool, issues: Vec<String>, json: bool) -> Result<()> {
    if json {
        let output = ValidateJson {
            name: name.to_string(),
            valid,
            issues: issues.clone(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else if valid {
        println!("Schema '{}' is valid", name);
    } else {
        println!("Schema '{}' has errors:", name);
        for issue in &issues {
            println!("  error: {}", issue);
        }
    }

    if valid {
        Ok(())
    } else {
        Err(OpenSpecError::Custom(format!(
            "Schema '{}' is invalid",
            name
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_driven_is_valid() {
        let resolved = resolve_schema("spec-driven", None).unwrap();
        assert!(resolved.schema.validate().is_ok());
    }

    #[test]
    fn test_resolve_with_shadows_embedded() {
        // With no project root, spec-driven falls back to the embedded schema
        // when no on-disk package copy is present.
        let dir = std::path::PathBuf::from("/nonexistent-project-root-xyz");
        let resolution = resolve_with_shadows("spec-driven", &dir);
        assert!(resolution.is_some());
        let resolution = resolution.unwrap();
        assert_eq!(resolution.name, "spec-driven");
        assert_eq!(resolution.source, "package");
    }

    #[test]
    fn test_resolve_with_shadows_missing() {
        let dir = std::path::PathBuf::from("/nonexistent-project-root-xyz");
        assert!(resolve_with_shadows("does-not-exist", &dir).is_none());
    }

    #[test]
    fn test_is_valid_schema_name() {
        assert!(is_valid_schema_name("my-workflow"));
        assert!(is_valid_schema_name("flow1"));
        assert!(!is_valid_schema_name("Bad Name"));
        assert!(!is_valid_schema_name("Bad"));
        assert!(!is_valid_schema_name("-leading"));
        assert!(!is_valid_schema_name("trailing-"));
        assert!(!is_valid_schema_name("double--hyphen"));
    }

    // The fork/init commands operate relative to the current working directory.
    // A process-wide mutex serializes the tests that chdir so they don't race.
    static CWD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn fork_into(dir: &Path, source: &str, name: Option<&str>, force: bool) -> Result<()> {
        let _guard = CWD_LOCK.lock().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir).unwrap();
        let result = run_fork(source, name, force, true);
        std::env::set_current_dir(prev).unwrap();
        result
    }

    fn init_into(dir: &Path, name: &str, force: bool) -> Result<()> {
        let _guard = CWD_LOCK.lock().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir).unwrap();
        let result = run_init(name, force, true);
        std::env::set_current_dir(prev).unwrap();
        result
    }

    #[test]
    fn test_fork_embedded_spec_driven() {
        let tmp = std::env::temp_dir().join(format!("osfork-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        fork_into(&tmp, "spec-driven", Some("my-flow"), false).unwrap();

        let dest = tmp.join("openspec").join("schemas").join("my-flow");
        let schema_path = dest.join("schema.yaml");
        assert!(schema_path.exists());
        assert!(dest.join("templates").join("proposal.md").exists());
        assert!(dest.join("templates").join("spec.md").exists());
        assert!(dest.join("templates").join("design.md").exists());
        assert!(dest.join("templates").join("tasks.md").exists());

        let schema = load_schema(&schema_path).unwrap();
        assert_eq!(schema.name, "my-flow");
        assert!(schema.validate().is_ok());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_fork_invalid_name_errors() {
        let tmp = std::env::temp_dir().join(format!("osforkbad-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let result = fork_into(&tmp, "spec-driven", Some("Bad Name"), false);
        assert!(result.is_err());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_fork_existing_without_force_errors_with_force_overwrites() {
        let tmp = std::env::temp_dir().join(format!("osforkdup-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        fork_into(&tmp, "spec-driven", Some("dup-flow"), false).unwrap();
        // Second fork without --force should error.
        assert!(fork_into(&tmp, "spec-driven", Some("dup-flow"), false).is_err());
        // With --force it should overwrite and succeed.
        fork_into(&tmp, "spec-driven", Some("dup-flow"), true).unwrap();

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_init_scaffold_validates() {
        let tmp = std::env::temp_dir().join(format!("osinit-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        init_into(&tmp, "another-flow", false).unwrap();

        let dest = tmp.join("openspec").join("schemas").join("another-flow");
        let schema_path = dest.join("schema.yaml");
        assert!(schema_path.exists());
        assert!(dest.join("templates").join("proposal.md").exists());
        assert!(dest.join("templates").join("spec.md").exists());

        let schema = load_schema(&schema_path).unwrap();
        assert_eq!(schema.name, "another-flow");
        assert!(schema.validate().is_ok());

        // Existing without force errors.
        assert!(init_into(&tmp, "another-flow", false).is_err());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_init_invalid_name_errors() {
        let tmp = std::env::temp_dir().join(format!("osinitbad-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        assert!(init_into(&tmp, "Bad Name", false).is_err());

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
