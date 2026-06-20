use super::schema::{
    InitiativeState, INITIATIVE_DECISIONS_FILE_NAME, INITIATIVE_DESIGN_FILE_NAME,
    INITIATIVE_MARKDOWN_FILE_NAMES, INITIATIVE_QUESTIONS_FILE_NAME,
    INITIATIVE_REQUIREMENTS_FILE_NAME, INITIATIVE_TASKS_FILE_NAME,
};

/// A template file with its name and content.
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateFile {
    pub file_name: String,
    pub content: String,
}

/// Ensure content ends with a newline.
fn with_trailing_newline(content: &str) -> String {
    if content.ends_with('\n') {
        content.to_string()
    } else {
        format!("{}\n", content)
    }
}

/// Build the requirements.md template.
pub fn build_requirements(state: &InitiativeState) -> String {
    with_trailing_newline(&format!(
        "# Requirements

## Product Intent

{}

## Accepted Requirements

- TBD

## Out Of Scope

- TBD
",
        state.summary
    ))
}

/// Build the design.md template.
pub fn build_design(state: &InitiativeState) -> String {
    with_trailing_newline(&format!(
        "# Design

## Context

{}

## Approach

TBD

## Affected Areas

- TBD

## Dependencies

- TBD

## Risks

- TBD
",
        state.summary
    ))
}

/// Build the decisions.md template.
pub fn build_decisions(state: &InitiativeState) -> String {
    with_trailing_newline(&format!(
        "# Decisions

## Accepted Decisions

### {}: {}

- Decision: TBD
- Why: TBD
- Implications: TBD
",
        state.created, state.title
    ))
}

/// Build the questions.md template.
pub fn build_questions() -> String {
    with_trailing_newline(
        "# Questions

## Open Questions

- TBD

## Resolved Questions

- TBD
",
    )
}

/// Build the tasks.md template.
pub fn build_tasks() -> String {
    with_trailing_newline(
        "# Tasks

## Coordination Tasks

- [ ] TBD
",
    )
}

/// Build the default set of template files for an initiative.
/// Returns files in the order of INITIATIVE_MARKDOWN_FILE_NAMES.
pub fn build_default_initiative_files(state: &InitiativeState) -> Vec<TemplateFile> {
    let mut files = Vec::new();

    for &file_name in INITIATIVE_MARKDOWN_FILE_NAMES {
        let content = match file_name {
            INITIATIVE_REQUIREMENTS_FILE_NAME => build_requirements(state),
            INITIATIVE_DESIGN_FILE_NAME => build_design(state),
            INITIATIVE_DECISIONS_FILE_NAME => build_decisions(state),
            INITIATIVE_QUESTIONS_FILE_NAME => build_questions(),
            INITIATIVE_TASKS_FILE_NAME => build_tasks(),
            _ => unreachable!(),
        };

        files.push(TemplateFile {
            file_name: file_name.to_string(),
            content,
        });
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::collections::initiatives::InitiativeStatus;
    use std::collections::BTreeMap;

    #[test]
    fn test_build_requirements() {
        let state = InitiativeState {
            version: 1,
            id: "test".to_string(),
            title: "Test".to_string(),
            summary: "Test summary".to_string(),
            status: InitiativeStatus::Exploring,
            created: "2024-01-15".to_string(),
            owners: vec![],
            metadata: BTreeMap::new(),
        };

        let content = build_requirements(&state);
        assert!(content.contains("# Requirements"));
        assert!(content.contains("Test summary"));
        assert!(content.ends_with('\n'));
    }

    #[test]
    fn test_build_design() {
        let state = InitiativeState {
            version: 1,
            id: "test".to_string(),
            title: "Test".to_string(),
            summary: "Design summary".to_string(),
            status: InitiativeStatus::Exploring,
            created: "2024-01-15".to_string(),
            owners: vec![],
            metadata: BTreeMap::new(),
        };

        let content = build_design(&state);
        assert!(content.contains("# Design"));
        assert!(content.contains("Design summary"));
        assert!(content.ends_with('\n'));
    }

    #[test]
    fn test_build_decisions() {
        let state = InitiativeState {
            version: 1,
            id: "test".to_string(),
            title: "My Decision".to_string(),
            summary: "Summary".to_string(),
            status: InitiativeStatus::Exploring,
            created: "2024-01-15".to_string(),
            owners: vec![],
            metadata: BTreeMap::new(),
        };

        let content = build_decisions(&state);
        assert!(content.contains("# Decisions"));
        assert!(content.contains("2024-01-15: My Decision"));
        assert!(content.ends_with('\n'));
    }

    #[test]
    fn test_build_questions() {
        let content = build_questions();
        assert!(content.contains("# Questions"));
        assert!(content.contains("Open Questions"));
        assert!(content.ends_with('\n'));
    }

    #[test]
    fn test_build_tasks() {
        let content = build_tasks();
        assert!(content.contains("# Tasks"));
        assert!(content.contains("Coordination Tasks"));
        assert!(content.ends_with('\n'));
    }

    #[test]
    fn test_build_default_initiative_files() {
        let state = InitiativeState {
            version: 1,
            id: "test".to_string(),
            title: "Test Initiative".to_string(),
            summary: "A test initiative".to_string(),
            status: InitiativeStatus::Exploring,
            created: "2024-01-15".to_string(),
            owners: vec![],
            metadata: BTreeMap::new(),
        };

        let files = build_default_initiative_files(&state);

        // Should have all 5 template files
        assert_eq!(files.len(), 5);

        // Check order
        assert_eq!(files[0].file_name, "requirements.md");
        assert_eq!(files[1].file_name, "design.md");
        assert_eq!(files[2].file_name, "decisions.md");
        assert_eq!(files[3].file_name, "questions.md");
        assert_eq!(files[4].file_name, "tasks.md");

        // All should end with newline
        for file in &files {
            assert!(file.content.ends_with('\n'));
        }
    }
}
