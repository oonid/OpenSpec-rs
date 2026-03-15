use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::core::error::{OpenSpecError, Result};

const EMBEDDED_SPEC_DRIVEN_SCHEMA: &str = include_str!("../embedded_schemas/spec-driven.yaml");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Artifact {
    pub id: String,
    pub generates: String,
    pub description: String,
    pub template: String,
    #[serde(default)]
    pub instruction: Option<String>,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApplyPhase {
    pub requires: Vec<String>,
    #[serde(default)]
    pub tracks: Option<String>,
    #[serde(default)]
    pub instruction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchemaYaml {
    pub name: String,
    pub version: u32,
    #[serde(default)]
    pub description: Option<String>,
    pub artifacts: Vec<Artifact>,
    #[serde(default)]
    pub apply: Option<ApplyPhase>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangeMetadata {
    pub schema: String,
    #[serde(default)]
    pub created: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedSchema {
    pub schema: SchemaYaml,
    pub path: String,
    pub source: SchemaSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaSource {
    Project,
    User,
    Package,
}

impl SchemaYaml {
    pub fn artifact_by_id(&self, id: &str) -> Option<&Artifact> {
        self.artifacts.iter().find(|a| a.id == id)
    }

    pub fn artifact_ids(&self) -> Vec<&str> {
        self.artifacts.iter().map(|a| a.id.as_str()).collect()
    }

    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.name.is_empty() {
            return Err("Schema name is required".to_string());
        }
        if self.version == 0 {
            return Err("Version must be a positive integer".to_string());
        }
        if self.artifacts.is_empty() {
            return Err("At least one artifact required".to_string());
        }
        for artifact in &self.artifacts {
            if artifact.id.is_empty() {
                return Err("Artifact ID is required".to_string());
            }
            if artifact.generates.is_empty() {
                return Err(format!(
                    "Artifact '{}' requires 'generates' field",
                    artifact.id
                ));
            }
            if artifact.template.is_empty() {
                return Err(format!(
                    "Artifact '{}' requires 'template' field",
                    artifact.id
                ));
            }
        }
        let ids: HashSet<&str> = self.artifacts.iter().map(|a| a.id.as_str()).collect();
        if ids.len() != self.artifacts.len() {
            return Err("Duplicate artifact IDs found".to_string());
        }
        for artifact in &self.artifacts {
            for req in &artifact.requires {
                if !ids.contains(req.as_str()) {
                    return Err(format!(
                        "Artifact '{}' requires non-existent artifact '{}'",
                        artifact.id, req
                    ));
                }
            }
        }
        if let Some(ref apply) = self.apply {
            for req in &apply.requires {
                if !ids.contains(req.as_str()) {
                    return Err(format!(
                        "Apply phase requires non-existent artifact '{}'",
                        req
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn topological_order(&self) -> Vec<&str> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp = HashSet::new();
        let artifact_map: HashMap<&str, &Artifact> =
            self.artifacts.iter().map(|a| (a.id.as_str(), a)).collect();

        fn visit<'a>(
            id: &'a str,
            artifact_map: &HashMap<&'a str, &'a Artifact>,
            visited: &mut HashSet<&'a str>,
            temp: &mut HashSet<&'a str>,
            result: &mut Vec<&'a str>,
        ) {
            if visited.contains(id) {
                return;
            }
            if temp.contains(id) {
                return;
            }
            temp.insert(id);
            if let Some(artifact) = artifact_map.get(id) {
                for req in &artifact.requires {
                    visit(req.as_str(), artifact_map, visited, temp, result);
                }
            }
            temp.remove(id);
            visited.insert(id);
            result.push(id);
        }

        for artifact in &self.artifacts {
            visit(
                &artifact.id,
                &artifact_map,
                &mut visited,
                &mut temp,
                &mut result,
            );
        }

        result
    }
}

pub fn load_schema<P: AsRef<Path>>(path: P) -> Result<SchemaYaml> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path).map_err(|e| OpenSpecError::IoRead {
        path: path.to_path_buf(),
        source: e,
    })?;
    parse_schema(&content, path.display().to_string())
}

pub fn parse_schema(yaml_content: &str, context: String) -> Result<SchemaYaml> {
    let schema: SchemaYaml = serde_yaml::from_str(yaml_content)
        .map_err(|e| OpenSpecError::yaml_parse(&context, e.to_string()))?;
    schema
        .validate()
        .map_err(|msg| OpenSpecError::SchemaValidation { message: msg })?;
    validate_no_cycles(&schema)?;
    Ok(schema)
}

fn validate_no_cycles(schema: &SchemaYaml) -> Result<()> {
    let artifact_map: HashMap<&str, &Artifact> = schema
        .artifacts
        .iter()
        .map(|a| (a.id.as_str(), a))
        .collect();
    let mut visited = HashSet::new();
    let mut in_stack = HashSet::new();

    fn dfs<'a>(
        id: &'a str,
        artifact_map: &HashMap<&'a str, &'a Artifact>,
        visited: &mut HashSet<&'a str>,
        in_stack: &mut HashSet<&'a str>,
    ) -> Option<String> {
        visited.insert(id);
        in_stack.insert(id);

        if let Some(artifact) = artifact_map.get(id) {
            for dep in &artifact.requires {
                if !visited.contains(dep.as_str()) {
                    if let Some(cycle) = dfs(dep.as_str(), artifact_map, visited, in_stack) {
                        return Some(cycle);
                    }
                } else if in_stack.contains(dep.as_str()) {
                    return Some(format!("{} -> {}", dep, id));
                }
            }
        }

        in_stack.remove(id);
        None
    }

    for artifact in &schema.artifacts {
        if !visited.contains(artifact.id.as_str()) {
            if let Some(cycle) = dfs(
                artifact.id.as_str(),
                &artifact_map,
                &mut visited,
                &mut in_stack,
            ) {
                return Err(OpenSpecError::CyclicDependency { cycle });
            }
        }
    }

    Ok(())
}

pub fn get_project_schemas_dir(project_root: &Path) -> std::path::PathBuf {
    project_root.join("openspec").join("schemas")
}

pub fn get_user_schemas_dir() -> std::path::PathBuf {
    crate::core::config::xdg_data_dir().join("schemas")
}

pub fn get_package_schemas_dir() -> std::path::PathBuf {
    std::path::PathBuf::from("vendor/OpenSpec/schemas")
}

pub fn get_embedded_spec_driven_schema() -> Result<SchemaYaml> {
    parse_schema(
        EMBEDDED_SPEC_DRIVEN_SCHEMA,
        "embedded:spec-driven.yaml".to_string(),
    )
}

pub fn resolve_schema(name: &str, project_root: Option<&Path>) -> Result<ResolvedSchema> {
    if let Some(root) = project_root {
        let project_dir = get_project_schemas_dir(root);
        let project_schema_path = project_dir.join(name).join("schema.yaml");
        if project_schema_path.exists() {
            let schema = load_schema(&project_schema_path)?;
            return Ok(ResolvedSchema {
                schema,
                path: project_schema_path.display().to_string(),
                source: SchemaSource::Project,
            });
        }
    }

    let user_dir = get_user_schemas_dir();
    let user_schema_path = user_dir.join(name).join("schema.yaml");
    if user_schema_path.exists() {
        let schema = load_schema(&user_schema_path)?;
        return Ok(ResolvedSchema {
            schema,
            path: user_schema_path.display().to_string(),
            source: SchemaSource::User,
        });
    }

    let package_dir = get_package_schemas_dir();
    let package_schema_path = package_dir.join(name).join("schema.yaml");
    if package_schema_path.exists() {
        let schema = load_schema(&package_schema_path)?;
        return Ok(ResolvedSchema {
            schema,
            path: package_schema_path.display().to_string(),
            source: SchemaSource::Package,
        });
    }

    if name == "spec-driven" {
        let schema = get_embedded_spec_driven_schema()?;
        return Ok(ResolvedSchema {
            schema,
            path: "embedded:spec-driven.yaml".to_string(),
            source: SchemaSource::Package,
        });
    }

    Err(OpenSpecError::schema_not_found(
        name,
        list_schema_names(project_root).join(", "),
    ))
}

#[derive(Debug, Clone)]
pub struct SchemaInfo {
    pub name: String,
    pub description: String,
    pub artifacts: Vec<String>,
    pub source: SchemaSource,
}

pub fn list_schemas(project_root: Option<&Path>) -> Vec<SchemaInfo> {
    let mut schemas: Vec<SchemaInfo> = Vec::new();
    let mut seen_names: HashSet<String> = HashSet::new();

    if let Some(root) = project_root {
        let project_dir = get_project_schemas_dir(root);
        if project_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&project_dir) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let schema_path = entry.path().join("schema.yaml");
                        if schema_path.exists() {
                            if let Ok(schema) = load_schema(&schema_path) {
                                schemas.push(SchemaInfo {
                                    name: name.clone(),
                                    description: schema.description.clone().unwrap_or_default(),
                                    artifacts: schema
                                        .artifacts
                                        .iter()
                                        .map(|a| a.id.clone())
                                        .collect(),
                                    source: SchemaSource::Project,
                                });
                                seen_names.insert(name);
                            }
                        }
                    }
                }
            }
        }
    }

    let user_dir = get_user_schemas_dir();
    if user_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&user_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !seen_names.contains(&name) {
                        let schema_path = entry.path().join("schema.yaml");
                        if schema_path.exists() {
                            if let Ok(schema) = load_schema(&schema_path) {
                                schemas.push(SchemaInfo {
                                    name: name.clone(),
                                    description: schema.description.clone().unwrap_or_default(),
                                    artifacts: schema
                                        .artifacts
                                        .iter()
                                        .map(|a| a.id.clone())
                                        .collect(),
                                    source: SchemaSource::User,
                                });
                                seen_names.insert(name);
                            }
                        }
                    }
                }
            }
        }
    }

    let package_dir = get_package_schemas_dir();
    if package_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&package_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !seen_names.contains(&name) {
                        let schema_path = entry.path().join("schema.yaml");
                        if schema_path.exists() {
                            if let Ok(schema) = load_schema(&schema_path) {
                                schemas.push(SchemaInfo {
                                    name: name.clone(),
                                    description: schema.description.clone().unwrap_or_default(),
                                    artifacts: schema
                                        .artifacts
                                        .iter()
                                        .map(|a| a.id.clone())
                                        .collect(),
                                    source: SchemaSource::Package,
                                });
                                seen_names.insert(name);
                            }
                        }
                    }
                }
            }
        }
    }

    if !seen_names.contains("spec-driven") {
        if let Ok(schema) = get_embedded_spec_driven_schema() {
            schemas.push(SchemaInfo {
                name: "spec-driven".to_string(),
                description: schema.description.clone().unwrap_or_default(),
                artifacts: schema.artifacts.iter().map(|a| a.id.clone()).collect(),
                source: SchemaSource::Package,
            });
        }
    }

    schemas.sort_by(|a, b| a.name.cmp(&b.name));
    schemas
}

pub fn list_schema_names(project_root: Option<&Path>) -> Vec<String> {
    list_schemas(project_root)
        .into_iter()
        .map(|s| s.name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_validation() {
        let schema = SchemaYaml {
            name: "test".to_string(),
            version: 1,
            description: Some("Test schema".to_string()),
            artifacts: vec![Artifact {
                id: "proposal".to_string(),
                generates: "proposal.md".to_string(),
                description: "Proposal".to_string(),
                template: "proposal.md".to_string(),
                instruction: None,
                requires: vec![],
            }],
            apply: None,
        };
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn test_schema_missing_name() {
        let schema = SchemaYaml {
            name: "".to_string(),
            version: 1,
            description: None,
            artifacts: vec![],
            apply: None,
        };
        assert!(schema.validate().is_err());
    }

    #[test]
    fn test_topological_order() {
        let schema = SchemaYaml {
            name: "test".to_string(),
            version: 1,
            description: None,
            artifacts: vec![
                Artifact {
                    id: "tasks".to_string(),
                    generates: "tasks.md".to_string(),
                    description: "Tasks".to_string(),
                    template: "tasks.md".to_string(),
                    instruction: None,
                    requires: vec!["specs".to_string()],
                },
                Artifact {
                    id: "proposal".to_string(),
                    generates: "proposal.md".to_string(),
                    description: "Proposal".to_string(),
                    template: "proposal.md".to_string(),
                    instruction: None,
                    requires: vec![],
                },
                Artifact {
                    id: "specs".to_string(),
                    generates: "specs/**/*.md".to_string(),
                    description: "Specs".to_string(),
                    template: "spec.md".to_string(),
                    instruction: None,
                    requires: vec!["proposal".to_string()],
                },
            ],
            apply: None,
        };
        let order = schema.topological_order();
        let proposal_idx = order.iter().position(|&id| id == "proposal").unwrap();
        let specs_idx = order.iter().position(|&id| id == "specs").unwrap();
        let tasks_idx = order.iter().position(|&id| id == "tasks").unwrap();
        assert!(proposal_idx < specs_idx);
        assert!(specs_idx < tasks_idx);
    }

    #[test]
    fn test_parse_schema_yaml() {
        let yaml = r#"
name: test-schema
version: 1
description: Test schema
artifacts:
  - id: proposal
    generates: proposal.md
    description: Proposal document
    template: proposal.md
    requires: []
"#;
        let schema = parse_schema(yaml, "test".to_string()).unwrap();
        assert_eq!(schema.name, "test-schema");
        assert_eq!(schema.version, 1);
        assert_eq!(schema.artifacts.len(), 1);
        assert_eq!(schema.artifacts[0].id, "proposal");
    }

    #[test]
    fn test_parse_schema_with_dependencies() {
        let yaml = r#"
name: workflow
version: 1
artifacts:
  - id: proposal
    generates: proposal.md
    description: Proposal
    template: proposal.md
  - id: specs
    generates: specs/**/*.md
    description: Specs
    template: spec.md
    requires:
      - proposal
"#;
        let schema = parse_schema(yaml, "test".to_string()).unwrap();
        assert_eq!(schema.artifacts.len(), 2);
        let specs = schema.artifact_by_id("specs").unwrap();
        assert_eq!(specs.requires, vec!["proposal"]);
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let yaml = "not: valid: yaml:::";
        let result = parse_schema(yaml, "test".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_required_field() {
        let yaml = r#"
name: test
version: 1
artifacts:
  - id: proposal
    description: Missing generates and template
"#;
        let result = parse_schema(yaml, "test".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_schema_file() {
        let schema_path = "vendor/OpenSpec/schemas/spec-driven/schema.yaml";
        if !std::path::Path::new(schema_path).exists() {
            eprintln!("Skipping test_load_schema_file: vendor directory not available");
            return;
        }
        let result = load_schema(schema_path);
        assert!(result.is_ok());
        let schema = result.unwrap();
        assert_eq!(schema.name, "spec-driven");
        assert!(!schema.artifacts.is_empty());
    }

    #[test]
    fn test_detect_cycle() {
        let yaml = r#"
name: cyclic
version: 1
artifacts:
  - id: a
    generates: a.md
    description: A
    template: a.md
    requires:
      - b
  - id: b
    generates: b.md
    description: B
    template: b.md
    requires:
      - a
"#;
        let result = parse_schema(yaml, "test".to_string());
        assert!(result.is_err());
        match result {
            Err(OpenSpecError::CyclicDependency { .. }) => {}
            _ => panic!("Expected CyclicDependency error"),
        }
    }

    #[test]
    fn test_resolve_schema_package() {
        let result = resolve_schema("spec-driven", None);
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert_eq!(resolved.schema.name, "spec-driven");
        assert_eq!(resolved.source, SchemaSource::Package);
    }

    #[test]
    fn test_resolve_schema_not_found() {
        let result = resolve_schema("nonexistent-schema", None);
        assert!(result.is_err());
        match result {
            Err(OpenSpecError::SchemaNotFound { name, .. }) => {
                assert_eq!(name, "nonexistent-schema");
            }
            _ => panic!("Expected SchemaNotFound error"),
        }
    }

    #[test]
    fn test_list_schemas() {
        let schemas = list_schemas(None);
        assert!(!schemas.is_empty());
        let spec_driven = schemas.iter().find(|s| s.name == "spec-driven");
        assert!(spec_driven.is_some());
        let schema = spec_driven.unwrap();
        assert_eq!(schema.source, SchemaSource::Package);
        assert!(!schema.artifacts.is_empty());
    }

    #[test]
    fn test_list_schema_names() {
        let names = list_schema_names(None);
        assert!(!names.is_empty());
        assert!(names.contains(&"spec-driven".to_string()));
    }

    #[test]
    fn test_embedded_spec_driven_schema() {
        let schema = get_embedded_spec_driven_schema().unwrap();
        assert_eq!(schema.name, "spec-driven");
        assert_eq!(schema.version, 1);
        assert!(!schema.artifacts.is_empty());
        let proposal = schema.artifact_by_id("proposal").unwrap();
        assert_eq!(proposal.generates, "proposal.md");
        assert!(schema.apply.is_some());
    }
}
