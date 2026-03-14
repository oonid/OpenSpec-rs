use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;

use crate::core::schema::{Artifact, SchemaYaml};

pub type CompletedSet = HashSet<String>;
pub type BlockedArtifacts = HashMap<String, Vec<String>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactStatusKind {
    Done,
    Ready,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactStatus {
    pub id: String,
    pub output_path: String,
    pub status: ArtifactStatusKind,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub missing_deps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeStatus {
    pub change_name: String,
    pub schema_name: String,
    pub is_complete: bool,
    pub apply_requires: Vec<String>,
    pub artifacts: Vec<ArtifactStatus>,
}

impl ChangeStatus {
    pub fn format_text(&self, writer: &mut impl Write) -> std::io::Result<()> {
        let done_count = self
            .artifacts
            .iter()
            .filter(|a| a.status == ArtifactStatusKind::Done)
            .count();
        let total = self.artifacts.len();

        writeln!(writer, "Change: {}", self.change_name)?;
        writeln!(writer, "Schema: {}", self.schema_name)?;
        writeln!(
            writer,
            "Progress: {}/{} artifacts complete",
            done_count, total
        )?;
        writeln!(writer)?;

        for artifact in &self.artifacts {
            let indicator = match artifact.status {
                ArtifactStatusKind::Done => "[x]",
                ArtifactStatusKind::Ready => "[ ]",
                ArtifactStatusKind::Blocked => "[-]",
            };

            if artifact.status == ArtifactStatusKind::Blocked && !artifact.missing_deps.is_empty() {
                let deps = artifact.missing_deps.join(", ");
                writeln!(writer, "{} {} (blocked: {})", indicator, artifact.id, deps)?;
            } else {
                writeln!(writer, "{} {}", indicator, artifact.id)?;
            }
        }

        Ok(())
    }

    pub fn format_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Clone)]
pub struct DependencyInfo {
    pub id: String,
    pub done: bool,
    pub path: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ChangeContext {
    pub graph: ArtifactGraph,
    pub completed: CompletedSet,
    pub schema_name: String,
    pub change_name: String,
    pub change_dir: PathBuf,
    pub project_root: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ArtifactGraph {
    artifacts: HashMap<String, Artifact>,
}

impl ArtifactGraph {
    pub fn new(schema: &SchemaYaml) -> Self {
        let artifacts = schema
            .artifacts
            .iter()
            .map(|a| (a.id.clone(), a.clone()))
            .collect();
        Self { artifacts }
    }

    pub fn get_all_artifacts(&self) -> Vec<&Artifact> {
        self.artifacts.values().collect()
    }

    pub fn get_artifact(&self, id: &str) -> Option<&Artifact> {
        self.artifacts.get(id)
    }

    pub fn get_build_order(&self) -> Vec<String> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

        for artifact in self.artifacts.values() {
            in_degree.entry(artifact.id.as_str()).or_insert(0);
            dependents.entry(artifact.id.as_str()).or_default();

            for req in &artifact.requires {
                *in_degree.entry(artifact.id.as_str()).or_insert(0) += 1;
                dependents
                    .entry(req.as_str())
                    .or_default()
                    .push(&artifact.id);
            }
        }

        let mut queue: Vec<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();
        queue.sort();

        let mut result = Vec::new();

        while !queue.is_empty() {
            queue.sort();
            let current = queue.remove(0);
            result.push(current.to_string());

            if let Some(deps) = dependents.get(current) {
                for &dependent in deps {
                    if let Some(deg) = in_degree.get_mut(dependent) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(dependent);
                        }
                    }
                }
            }
        }

        result
    }

    pub fn get_next_artifacts(&self, completed: &CompletedSet) -> Vec<String> {
        let mut ready = Vec::new();

        for artifact in self.artifacts.values() {
            if completed.contains(&artifact.id) {
                continue;
            }

            let all_deps_completed = artifact.requires.iter().all(|req| completed.contains(req));
            if all_deps_completed {
                ready.push(artifact.id.clone());
            }
        }

        ready.sort();
        ready
    }

    pub fn get_blocked(&self, completed: &CompletedSet) -> BlockedArtifacts {
        let mut blocked = BlockedArtifacts::new();

        for artifact in self.artifacts.values() {
            if completed.contains(&artifact.id) {
                continue;
            }

            let unmet_deps: Vec<String> = artifact
                .requires
                .iter()
                .filter(|req| !completed.contains(*req))
                .cloned()
                .collect();

            if !unmet_deps.is_empty() {
                let mut sorted_deps = unmet_deps;
                sorted_deps.sort();
                blocked.insert(artifact.id.clone(), sorted_deps);
            }
        }

        blocked
    }
}

impl ChangeContext {
    pub fn new(
        schema: &SchemaYaml,
        completed: CompletedSet,
        change_name: &str,
        change_dir: PathBuf,
        project_root: Option<PathBuf>,
    ) -> Self {
        Self {
            graph: ArtifactGraph::new(schema),
            completed,
            schema_name: schema.name.clone(),
            change_name: change_name.to_string(),
            change_dir,
            project_root,
        }
    }

    pub fn compute_status(&self, schema: &SchemaYaml) -> ChangeStatus {
        let apply_requires: Vec<String> = schema
            .apply
            .as_ref()
            .map(|a| a.requires.clone())
            .unwrap_or_else(|| schema.artifacts.iter().map(|a| a.id.clone()).collect());

        let ready: HashSet<String> = self
            .graph
            .get_next_artifacts(&self.completed)
            .into_iter()
            .collect();
        let blocked = self.graph.get_blocked(&self.completed);

        let artifact_statuses: Vec<ArtifactStatus> = self
            .graph
            .get_all_artifacts()
            .iter()
            .map(|artifact| {
                if self.completed.contains(&artifact.id) {
                    ArtifactStatus {
                        id: artifact.id.clone(),
                        output_path: artifact.generates.clone(),
                        status: ArtifactStatusKind::Done,
                        missing_deps: vec![],
                    }
                } else if ready.contains(&artifact.id) {
                    ArtifactStatus {
                        id: artifact.id.clone(),
                        output_path: artifact.generates.clone(),
                        status: ArtifactStatusKind::Ready,
                        missing_deps: vec![],
                    }
                } else {
                    let missing = blocked.get(&artifact.id).cloned().unwrap_or_default();
                    ArtifactStatus {
                        id: artifact.id.clone(),
                        output_path: artifact.generates.clone(),
                        status: ArtifactStatusKind::Blocked,
                        missing_deps: missing,
                    }
                }
            })
            .collect();

        let is_complete = self.graph.get_all_artifacts().len() == self.completed.len();

        ChangeStatus {
            change_name: self.change_name.clone(),
            schema_name: self.schema_name.clone(),
            is_complete,
            apply_requires,
            artifacts: artifact_statuses,
        }
    }

    pub fn get_dependency_info(&self, artifact_id: &str) -> Option<Vec<DependencyInfo>> {
        let artifact = self.graph.get_artifact(artifact_id)?;
        let deps: Vec<DependencyInfo> = artifact
            .requires
            .iter()
            .filter_map(|req_id| {
                self.graph.get_artifact(req_id).map(|dep| DependencyInfo {
                    id: dep.id.clone(),
                    done: self.completed.contains(&dep.id),
                    path: dep.generates.clone(),
                    description: dep.description.clone(),
                })
            })
            .collect();
        Some(deps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_schema() -> SchemaYaml {
        SchemaYaml {
            name: "test".to_string(),
            version: 1,
            description: None,
            artifacts: vec![
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
                Artifact {
                    id: "tasks".to_string(),
                    generates: "tasks.md".to_string(),
                    description: "Tasks".to_string(),
                    template: "tasks.md".to_string(),
                    instruction: None,
                    requires: vec!["specs".to_string()],
                },
            ],
            apply: None,
        }
    }

    #[test]
    fn test_graph_build_order() {
        let schema = make_test_schema();
        let graph = ArtifactGraph::new(&schema);
        let order = graph.get_build_order();

        assert_eq!(order.len(), 3);
        let proposal_idx = order.iter().position(|id| id == "proposal").unwrap();
        let specs_idx = order.iter().position(|id| id == "specs").unwrap();
        let tasks_idx = order.iter().position(|id| id == "tasks").unwrap();
        assert!(proposal_idx < specs_idx);
        assert!(specs_idx < tasks_idx);
    }

    #[test]
    fn test_get_next_artifacts_empty() {
        let schema = make_test_schema();
        let graph = ArtifactGraph::new(&schema);
        let completed = CompletedSet::new();

        let next = graph.get_next_artifacts(&completed);
        assert_eq!(next, vec!["proposal"]);
    }

    #[test]
    fn test_get_next_artifacts_partial() {
        let schema = make_test_schema();
        let graph = ArtifactGraph::new(&schema);
        let mut completed = CompletedSet::new();
        completed.insert("proposal".to_string());

        let next = graph.get_next_artifacts(&completed);
        assert_eq!(next, vec!["specs"]);
    }

    #[test]
    fn test_get_blocked() {
        let schema = make_test_schema();
        let graph = ArtifactGraph::new(&schema);
        let completed = CompletedSet::new();

        let blocked = graph.get_blocked(&completed);
        assert!(!blocked.contains_key("proposal"));
        assert!(blocked.contains_key("specs"));
        assert!(blocked.contains_key("tasks"));
        assert_eq!(blocked.get("specs"), Some(&vec!["proposal".to_string()]));
    }

    #[test]
    fn test_get_blocked_partial() {
        let schema = make_test_schema();
        let graph = ArtifactGraph::new(&schema);
        let mut completed = CompletedSet::new();
        completed.insert("proposal".to_string());

        let blocked = graph.get_blocked(&completed);
        assert!(!blocked.contains_key("proposal"));
        assert!(!blocked.contains_key("specs"));
        assert!(blocked.contains_key("tasks"));
        assert_eq!(blocked.get("tasks"), Some(&vec!["specs".to_string()]));
    }

    #[test]
    fn test_get_all_artifacts() {
        let schema = make_test_schema();
        let graph = ArtifactGraph::new(&schema);

        let artifacts = graph.get_all_artifacts();
        assert_eq!(artifacts.len(), 3);
    }

    #[test]
    fn test_get_artifact() {
        let schema = make_test_schema();
        let graph = ArtifactGraph::new(&schema);

        let proposal = graph.get_artifact("proposal");
        assert!(proposal.is_some());
        assert_eq!(proposal.unwrap().id, "proposal");

        let missing = graph.get_artifact("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_compute_status_empty() {
        let schema = make_test_schema();
        let completed = CompletedSet::new();
        let ctx = ChangeContext::new(
            &schema,
            completed,
            "test-change",
            PathBuf::from("/tmp/test"),
            None,
        );

        let status = ctx.compute_status(&schema);
        assert_eq!(status.change_name, "test-change");
        assert_eq!(status.schema_name, "test");
        assert!(!status.is_complete);
        assert_eq!(status.artifacts.len(), 3);

        let proposal_status = status
            .artifacts
            .iter()
            .find(|a| a.id == "proposal")
            .unwrap();
        assert_eq!(proposal_status.status, ArtifactStatusKind::Ready);

        let specs_status = status.artifacts.iter().find(|a| a.id == "specs").unwrap();
        assert_eq!(specs_status.status, ArtifactStatusKind::Blocked);
        assert_eq!(specs_status.missing_deps, vec!["proposal"]);
    }

    #[test]
    fn test_compute_status_partial() {
        let schema = make_test_schema();
        let mut completed = CompletedSet::new();
        completed.insert("proposal".to_string());
        let ctx = ChangeContext::new(
            &schema,
            completed,
            "test-change",
            PathBuf::from("/tmp/test"),
            None,
        );

        let status = ctx.compute_status(&schema);
        assert!(!status.is_complete);

        let proposal_status = status
            .artifacts
            .iter()
            .find(|a| a.id == "proposal")
            .unwrap();
        assert_eq!(proposal_status.status, ArtifactStatusKind::Done);

        let specs_status = status.artifacts.iter().find(|a| a.id == "specs").unwrap();
        assert_eq!(specs_status.status, ArtifactStatusKind::Ready);
        assert!(specs_status.missing_deps.is_empty());
    }

    #[test]
    fn test_compute_status_complete() {
        let schema = make_test_schema();
        let mut completed = CompletedSet::new();
        completed.insert("proposal".to_string());
        completed.insert("specs".to_string());
        completed.insert("tasks".to_string());
        let ctx = ChangeContext::new(
            &schema,
            completed,
            "test-change",
            PathBuf::from("/tmp/test"),
            None,
        );

        let status = ctx.compute_status(&schema);
        assert!(status.is_complete);

        for artifact_status in &status.artifacts {
            assert_eq!(artifact_status.status, ArtifactStatusKind::Done);
        }
    }

    #[test]
    fn test_apply_requires_from_schema() {
        let mut schema = make_test_schema();
        schema.apply = Some(crate::core::schema::ApplyPhase {
            requires: vec!["tasks".to_string()],
            tracks: None,
            instruction: None,
        });

        let completed = CompletedSet::new();
        let ctx = ChangeContext::new(
            &schema,
            completed,
            "test-change",
            PathBuf::from("/tmp/test"),
            None,
        );

        let status = ctx.compute_status(&schema);
        assert_eq!(status.apply_requires, vec!["tasks"]);
    }

    #[test]
    fn test_get_dependency_info() {
        let schema = make_test_schema();
        let mut completed = CompletedSet::new();
        completed.insert("proposal".to_string());
        let ctx = ChangeContext::new(
            &schema,
            completed,
            "test-change",
            PathBuf::from("/tmp/test"),
            None,
        );

        let deps = ctx.get_dependency_info("specs").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].id, "proposal");
        assert!(deps[0].done);
        assert_eq!(deps[0].path, "proposal.md");
    }

    #[test]
    fn test_format_status_text() {
        let schema = make_test_schema();
        let completed = CompletedSet::new();
        let ctx = ChangeContext::new(
            &schema,
            completed,
            "my-change",
            PathBuf::from("/tmp/test"),
            None,
        );

        let status = ctx.compute_status(&schema);
        let mut output = Vec::new();
        status.format_text(&mut output).unwrap();
        let text = String::from_utf8(output).unwrap();

        assert!(text.contains("Change: my-change"));
        assert!(text.contains("Schema: test"));
        assert!(text.contains("Progress: 0/3"));
        assert!(text.contains("[ ] proposal"));
        assert!(text.contains("[-] specs (blocked: proposal)"));
        assert!(text.contains("[-] tasks (blocked: specs)"));
    }

    #[test]
    fn test_format_status_text_partial() {
        let schema = make_test_schema();
        let mut completed = CompletedSet::new();
        completed.insert("proposal".to_string());
        let ctx = ChangeContext::new(
            &schema,
            completed,
            "my-change",
            PathBuf::from("/tmp/test"),
            None,
        );

        let status = ctx.compute_status(&schema);
        let mut output = Vec::new();
        status.format_text(&mut output).unwrap();
        let text = String::from_utf8(output).unwrap();

        assert!(text.contains("Progress: 1/3"));
        assert!(text.contains("[x] proposal"));
        assert!(text.contains("[ ] specs"));
        assert!(text.contains("[-] tasks (blocked: specs)"));
    }

    #[test]
    fn test_format_status_json() {
        let schema = make_test_schema();
        let completed = CompletedSet::new();
        let ctx = ChangeContext::new(
            &schema,
            completed,
            "my-change",
            PathBuf::from("/tmp/test"),
            None,
        );

        let status = ctx.compute_status(&schema);
        let json = status.format_json().unwrap();

        assert!(json.contains("\"change_name\": \"my-change\""));
        assert!(json.contains("\"schema_name\": \"test\""));
        assert!(json.contains("\"is_complete\": false"));
        assert!(json.contains("\"status\": \"ready\""));
        assert!(json.contains("\"status\": \"blocked\""));
        assert!(json.contains("\"missing_deps\""));
    }
}
