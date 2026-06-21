use std::collections::BTreeMap;
use std::path::Path;

use crate::core::error::{OpenSpecError, Result};
use crate::core::schema::{resolve_schema, ResolvedSchema};

const DEFAULT_SCHEMA: &str = "spec-driven";

/// Resolve the template reference for an artifact within a resolved schema.
///
/// When the schema resolves from disk (project/user/package), the template
/// lives under `<schemaDir>/templates/<template>`, so we emit that path. When
/// the schema is the embedded fallback (path `embedded:spec-driven.yaml`), no
/// filesystem path exists, so we emit the bare `template` value the schema
/// model carries (e.g. `proposal.md`).
fn template_reference(resolved: &ResolvedSchema, template: &str) -> String {
    if resolved.path.starts_with("embedded:") {
        return template.to_string();
    }

    let schema_path = Path::new(&resolved.path);
    match schema_path.parent() {
        Some(dir) => dir.join("templates").join(template).display().to_string(),
        None => template.to_string(),
    }
}

pub fn run(schema: Option<&str>, json: bool) -> Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    let schema_name = schema.unwrap_or(DEFAULT_SCHEMA);
    let resolved = resolve_schema(schema_name, Some(&project_root))?;

    let source = match resolved.source {
        crate::core::schema::SchemaSource::Project => "project",
        crate::core::schema::SchemaSource::User => "user",
        crate::core::schema::SchemaSource::Package => "package",
    };

    // Sorted by artifact id for deterministic output.
    let mut mapping: BTreeMap<String, String> = BTreeMap::new();
    for artifact in &resolved.schema.artifacts {
        mapping.insert(
            artifact.id.clone(),
            template_reference(&resolved, &artifact.template),
        );
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&mapping)?);
    } else {
        println!("Schema: {}", resolved.schema.name);
        println!("Source: {}", source);
        println!();
        for (id, template) in &mapping {
            println!("{}: {}", id, template);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::schema::get_embedded_spec_driven_schema;

    #[test]
    fn test_spec_driven_artifact_ids() {
        let resolved = resolve_schema("spec-driven", None).unwrap();
        let ids: Vec<&str> = resolved.schema.artifact_ids();
        for expected in ["proposal", "specs", "design", "tasks"] {
            assert!(
                ids.contains(&expected),
                "spec-driven should contain artifact '{}'",
                expected
            );
        }
    }

    #[test]
    fn test_template_reference_embedded() {
        let schema = get_embedded_spec_driven_schema().unwrap();
        let resolved = ResolvedSchema {
            schema,
            path: "embedded:spec-driven.yaml".to_string(),
            source: crate::core::schema::SchemaSource::Package,
        };
        assert_eq!(template_reference(&resolved, "proposal.md"), "proposal.md");
    }

    #[test]
    fn test_template_reference_disk() {
        let schema = get_embedded_spec_driven_schema().unwrap();
        let resolved = ResolvedSchema {
            schema,
            path: "vendor/OpenSpec/schemas/spec-driven/schema.yaml".to_string(),
            source: crate::core::schema::SchemaSource::Package,
        };
        assert_eq!(
            template_reference(&resolved, "proposal.md"),
            "vendor/OpenSpec/schemas/spec-driven/templates/proposal.md"
        );
    }
}
