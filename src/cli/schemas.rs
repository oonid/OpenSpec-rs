use crate::core::error::{OpenSpecError, Result};
use crate::core::schema::{list_schemas, SchemaSource};

#[derive(Debug, Clone, serde::Serialize)]
pub struct SchemaInfoJson {
    pub name: String,
    pub description: String,
    pub artifacts: Vec<String>,
    pub source: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SchemasOutput {
    pub schemas: Vec<SchemaInfoJson>,
}

pub fn run_schemas(json: bool) -> Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    let schemas = list_schemas(Some(&project_root));

    if json {
        let json_schemas: Vec<SchemaInfoJson> = schemas
            .iter()
            .map(|s| {
                let source = match s.source {
                    SchemaSource::Project => "project",
                    SchemaSource::User => "user",
                    SchemaSource::Package => "package",
                };
                SchemaInfoJson {
                    name: s.name.clone(),
                    description: s.description.clone(),
                    artifacts: s.artifacts.clone(),
                    source: source.to_string(),
                }
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&SchemasOutput {
                schemas: json_schemas,
            })?
        );
    } else {
        println!("Available schemas:");
        println!();

        for schema in &schemas {
            let source_label = match schema.source {
                SchemaSource::Project => " (project)",
                SchemaSource::User => " (user override)",
                SchemaSource::Package => "",
            };
            println!("  {}{}", schema.name, source_label);
            println!("    {}", schema.description);
            println!("    Artifacts: {}", schema.artifacts.join(" → "));
            println!();
        }
    }

    Ok(())
}
