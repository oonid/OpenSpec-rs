use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandContent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct GeneratedCommand {
    pub path: String,
    pub file_content: String,
}

pub trait ToolCommandAdapter: Send + Sync {
    fn tool_id(&self) -> &'static str;
    fn get_file_path(&self, command_id: &str) -> String;
    fn format_file(&self, content: &CommandContent) -> String;
}

pub struct ClaudeAdapter;
pub struct CursorAdapter;
pub struct WindsurfAdapter;
pub struct OpenCodeAdapter;

impl ToolCommandAdapter for ClaudeAdapter {
    fn tool_id(&self) -> &'static str {
        "claude"
    }

    fn get_file_path(&self, command_id: &str) -> String {
        format!(".claude/commands/opsx-{}.md", command_id)
    }

    fn format_file(&self, content: &CommandContent) -> String {
        format!(
            r#"---
name: {}
description: {}
---

{}
"#,
            content.name, content.description, content.body
        )
    }
}

impl ToolCommandAdapter for CursorAdapter {
    fn tool_id(&self) -> &'static str {
        "cursor"
    }

    fn get_file_path(&self, command_id: &str) -> String {
        format!(".cursor/commands/opsx-{}.md", command_id)
    }

    fn format_file(&self, content: &CommandContent) -> String {
        format!(
            r#"---
name: {}
description: {}
tags: [{}]
---

{}
"#,
            content.name,
            content.description,
            content.tags.join(", "),
            content.body
        )
    }
}

impl ToolCommandAdapter for WindsurfAdapter {
    fn tool_id(&self) -> &'static str {
        "windsurf"
    }

    fn get_file_path(&self, command_id: &str) -> String {
        format!(".windsurf/commands/opsx-{}.md", command_id)
    }

    fn format_file(&self, content: &CommandContent) -> String {
        format!(
            r#"---
name: {}
description: {}
---

{}
"#,
            content.name, content.description, content.body
        )
    }
}

impl ToolCommandAdapter for OpenCodeAdapter {
    fn tool_id(&self) -> &'static str {
        "opencode"
    }

    fn get_file_path(&self, command_id: &str) -> String {
        format!(".opencode/commands/opsx-{}.md", command_id)
    }

    fn format_file(&self, content: &CommandContent) -> String {
        format!(
            r#"---
name: {}
description: {}
---

{}
"#,
            content.name, content.description, content.body
        )
    }
}

pub fn get_adapter(tool_id: &str) -> Option<Box<dyn ToolCommandAdapter>> {
    match tool_id {
        "claude" => Some(Box::new(ClaudeAdapter)),
        "cursor" => Some(Box::new(CursorAdapter)),
        "windsurf" => Some(Box::new(WindsurfAdapter)),
        "opencode" => Some(Box::new(OpenCodeAdapter)),
        _ => None,
    }
}

pub fn generate_command(
    content: &CommandContent,
    adapter: &dyn ToolCommandAdapter,
) -> GeneratedCommand {
    GeneratedCommand {
        path: adapter.get_file_path(&content.id),
        file_content: adapter.format_file(content),
    }
}

pub fn generate_commands(
    contents: &[CommandContent],
    adapter: &dyn ToolCommandAdapter,
) -> Vec<GeneratedCommand> {
    contents
        .iter()
        .map(|c| generate_command(c, adapter))
        .collect()
}

pub fn get_command_contents(workflow_filter: Option<&[&str]>) -> Vec<CommandContent> {
    let all_commands = get_all_commands();
    match workflow_filter {
        Some(filter) => all_commands
            .into_iter()
            .filter(|c| filter.contains(&c.id.as_str()))
            .collect(),
        None => all_commands,
    }
}

fn get_all_commands() -> Vec<CommandContent> {
    vec![
        CommandContent {
            id: "explore".to_string(),
            name: "opsx:explore".to_string(),
            description: "Enter explore mode - a thinking partner for exploring ideas, investigating problems, and clarifying requirements.".to_string(),
            category: "Workflow".to_string(),
            tags: vec!["workflow".to_string(), "explore".to_string()],
            body: include_str!("commands/explore.md").to_string(),
        },
        CommandContent {
            id: "propose".to_string(),
            name: "opsx:propose".to_string(),
            description: "Propose a new change with all artifacts generated in one step.".to_string(),
            category: "Workflow".to_string(),
            tags: vec!["workflow".to_string(), "propose".to_string()],
            body: include_str!("commands/propose.md").to_string(),
        },
        CommandContent {
            id: "apply".to_string(),
            name: "opsx:apply".to_string(),
            description: "Implement tasks from an OpenSpec change.".to_string(),
            category: "Workflow".to_string(),
            tags: vec!["workflow".to_string(), "apply".to_string()],
            body: include_str!("commands/apply.md").to_string(),
        },
        CommandContent {
            id: "archive".to_string(),
            name: "opsx:archive".to_string(),
            description: "Archive a completed change in the experimental workflow.".to_string(),
            category: "Workflow".to_string(),
            tags: vec!["workflow".to_string(), "archive".to_string()],
            body: include_str!("commands/archive.md").to_string(),
        },
    ]
}
