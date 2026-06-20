pub struct SkillTemplate {
    pub name: &'static str,
    pub description: &'static str,
    pub instructions: &'static str,
    pub license: &'static str,
    pub compatibility: &'static str,
    pub metadata: &'static SkillMetadata,
}

pub struct SkillMetadata {
    pub author: &'static str,
    pub version: &'static str,
}

pub struct SkillEntry {
    pub template: &'static SkillTemplate,
    pub dir_name: &'static str,
    pub workflow_id: &'static str,
}

pub const SKILL_METADATA: SkillMetadata = SkillMetadata {
    author: "OpenSpec",
    version: "1.0",
};

pub const SKILL_EXPLORE: SkillTemplate = SkillTemplate {
    name: "openspec-explore",
    description: "Explore mode - a thinking partner for exploring ideas, investigating problems, and clarifying requirements.",
    instructions: include_str!("../../templates/skills/explore.md"),
    license: "MIT",
    compatibility: "Requires openspec CLI.",
    metadata: &SKILL_METADATA,
};

pub const SKILL_PROPOSE: SkillTemplate = SkillTemplate {
    name: "openspec-propose",
    description: "Propose a new change with all artifacts generated in one step.",
    instructions: include_str!("../../templates/skills/propose.md"),
    license: "MIT",
    compatibility: "Requires openspec CLI.",
    metadata: &SKILL_METADATA,
};

pub const SKILL_APPLY_CHANGE: SkillTemplate = SkillTemplate {
    name: "openspec-apply-change",
    description: "Implement tasks from an OpenSpec change.",
    instructions: include_str!("../../templates/skills/apply-change.md"),
    license: "MIT",
    compatibility: "Requires openspec CLI.",
    metadata: &SKILL_METADATA,
};

pub const SKILL_ARCHIVE_CHANGE: SkillTemplate = SkillTemplate {
    name: "openspec-archive-change",
    description: "Archive a completed change in the experimental workflow.",
    instructions: include_str!("../../templates/skills/archive-change.md"),
    license: "MIT",
    compatibility: "Requires openspec CLI.",
    metadata: &SKILL_METADATA,
};

pub const SKILL_VERIFY_CHANGE: SkillTemplate = SkillTemplate {
    name: "openspec-verify-change",
    description: "Verify implementation matches change artifacts (specs, tasks, design).",
    instructions: include_str!("../../templates/skills/verify-change.md"),
    license: "MIT",
    compatibility: "Requires openspec CLI.",
    metadata: &SKILL_METADATA,
};

pub const SKILL_SYNC_SPECS: SkillTemplate = SkillTemplate {
    name: "openspec-sync-specs",
    description: "Sync delta specs from a change to main specs.",
    instructions: include_str!("../../templates/skills/sync-specs.md"),
    license: "MIT",
    compatibility: "Requires openspec CLI.",
    metadata: &SKILL_METADATA,
};

pub const SKILL_ENTRIES: &[SkillEntry] = &[
    SkillEntry {
        template: &SKILL_EXPLORE,
        dir_name: "openspec-explore",
        workflow_id: "explore",
    },
    SkillEntry {
        template: &SKILL_PROPOSE,
        dir_name: "openspec-propose",
        workflow_id: "propose",
    },
    SkillEntry {
        template: &SKILL_APPLY_CHANGE,
        dir_name: "openspec-apply-change",
        workflow_id: "apply",
    },
    SkillEntry {
        template: &SKILL_ARCHIVE_CHANGE,
        dir_name: "openspec-archive-change",
        workflow_id: "archive",
    },
    SkillEntry {
        template: &SKILL_VERIFY_CHANGE,
        dir_name: "openspec-verify-change",
        workflow_id: "verify",
    },
    SkillEntry {
        template: &SKILL_SYNC_SPECS,
        dir_name: "openspec-sync-specs",
        workflow_id: "sync",
    },
];

pub fn get_skill_templates(workflow_filter: Option<&[&str]>) -> Vec<&'static SkillEntry> {
    match workflow_filter {
        Some(filter) => SKILL_ENTRIES
            .iter()
            .filter(|e| filter.contains(&e.workflow_id))
            .collect(),
        None => SKILL_ENTRIES.iter().collect(),
    }
}

pub fn generate_skill_content(template: &SkillTemplate, version: &str) -> String {
    format!(
        r#"---
name: {}
description: {}
license: {}
compatibility: {}
metadata:
  author: {}
  version: "1.0"
  generatedBy: "OpenSpec v{}"
---

{}
"#,
        template.name,
        template.description,
        template.license,
        template.compatibility,
        template.metadata.author,
        version,
        template.instructions
    )
}

pub const CORE_WORKFLOWS: &[&str] = &["propose", "explore", "apply", "sync", "archive"];

pub fn get_profile_workflows(profile: &str, custom_workflows: &[String]) -> Vec<String> {
    if profile == "custom" {
        custom_workflows.to_vec()
    } else {
        CORE_WORKFLOWS.iter().map(|s| s.to_string()).collect()
    }
}

pub fn transform_to_hyphen_commands(text: &str) -> String {
    text.replace("/opsx:", "/opsx-")
}

pub fn generate_skill_content_transformed(
    template: &SkillTemplate,
    version: &str,
    transform: bool,
) -> String {
    let content = generate_skill_content(template, version);
    if transform {
        transform_to_hyphen_commands(&content)
    } else {
        content
    }
}

pub fn extract_generated_by_version(skill_file: &std::path::Path) -> Option<String> {
    use std::fs;

    if !skill_file.exists() {
        return None;
    }

    match fs::read_to_string(skill_file) {
        Ok(content) => {
            for line in content.lines() {
                if let Some(value) = line.strip_prefix("  generatedBy:") {
                    let trimmed = value.trim().trim_matches('"').trim_matches('\'');
                    if let Some(version) = trimmed.strip_prefix("OpenSpec v") {
                        return Some(version.to_string());
                    }
                }
            }
            None
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_profile_workflows_core() {
        let workflows = get_profile_workflows("core", &[]);
        assert_eq!(
            workflows,
            vec!["propose", "explore", "apply", "sync", "archive"]
        );
    }

    #[test]
    fn test_get_profile_workflows_custom() {
        let custom = vec!["flow1".to_string(), "flow2".to_string()];
        let workflows = get_profile_workflows("custom", &custom);
        assert_eq!(workflows, custom);
    }

    #[test]
    fn test_transform_to_hyphen_commands() {
        assert_eq!(
            transform_to_hyphen_commands("/opsx:new"),
            "/opsx-new"
        );
        assert_eq!(
            transform_to_hyphen_commands("Use /opsx:apply to implement"),
            "Use /opsx-apply to implement"
        );
        assert_eq!(
            transform_to_hyphen_commands("/opsx:a /opsx:b"),
            "/opsx-a /opsx-b"
        );
    }

    #[test]
    fn test_extract_generated_by_version() {
        use tempfile::NamedTempFile;
        use std::io::Write;

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
    fn test_extract_generated_by_version_not_found() {
        use tempfile::NamedTempFile;
        use std::io::Write;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"---
name: test
---

content"#
        )
        .unwrap();

        let version = extract_generated_by_version(file.path());
        assert_eq!(version, None);
    }

    #[test]
    fn test_extract_generated_by_version_missing_file() {
        let version = extract_generated_by_version(std::path::Path::new("/nonexistent/file.md"));
        assert_eq!(version, None);
    }
}
