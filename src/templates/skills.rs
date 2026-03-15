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
