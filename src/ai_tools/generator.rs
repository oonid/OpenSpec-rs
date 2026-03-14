use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AITool {
    pub name: &'static str,
    pub value: &'static str,
    pub skills_dir: &'static str,
}

pub const AI_TOOLS: &[AITool] = &[
    AITool {
        name: "Amazon Q Developer",
        value: "amazon-q",
        skills_dir: ".amazonq",
    },
    AITool {
        name: "Antigravity",
        value: "antigravity",
        skills_dir: ".agent",
    },
    AITool {
        name: "Auggie (Augment CLI)",
        value: "auggie",
        skills_dir: ".augment",
    },
    AITool {
        name: "Claude Code",
        value: "claude",
        skills_dir: ".claude",
    },
    AITool {
        name: "Cline",
        value: "cline",
        skills_dir: ".cline",
    },
    AITool {
        name: "Codex",
        value: "codex",
        skills_dir: ".codex",
    },
    AITool {
        name: "CodeBuddy Code (CLI)",
        value: "codebuddy",
        skills_dir: ".codebuddy",
    },
    AITool {
        name: "Continue",
        value: "continue",
        skills_dir: ".continue",
    },
    AITool {
        name: "CoStrict",
        value: "costrict",
        skills_dir: ".cospec",
    },
    AITool {
        name: "Crush",
        value: "crush",
        skills_dir: ".crush",
    },
    AITool {
        name: "Cursor",
        value: "cursor",
        skills_dir: ".cursor",
    },
    AITool {
        name: "Factory Droid",
        value: "factory",
        skills_dir: ".factory",
    },
    AITool {
        name: "Gemini CLI",
        value: "gemini",
        skills_dir: ".gemini",
    },
    AITool {
        name: "GitHub Copilot",
        value: "github-copilot",
        skills_dir: ".github",
    },
    AITool {
        name: "iFlow",
        value: "iflow",
        skills_dir: ".iflow",
    },
    AITool {
        name: "Kilo Code",
        value: "kilocode",
        skills_dir: ".kilocode",
    },
    AITool {
        name: "Kiro",
        value: "kiro",
        skills_dir: ".kiro",
    },
    AITool {
        name: "OpenCode",
        value: "opencode",
        skills_dir: ".opencode",
    },
    AITool {
        name: "Pi",
        value: "pi",
        skills_dir: ".pi",
    },
    AITool {
        name: "Qoder",
        value: "qoder",
        skills_dir: ".qoder",
    },
    AITool {
        name: "Qwen Code",
        value: "qwen",
        skills_dir: ".qwen",
    },
    AITool {
        name: "RooCode",
        value: "roocode",
        skills_dir: ".roo",
    },
    AITool {
        name: "Trae",
        value: "trae",
        skills_dir: ".trae",
    },
    AITool {
        name: "Windsurf",
        value: "windsurf",
        skills_dir: ".windsurf",
    },
];

pub fn get_tools_with_skills_dir() -> Vec<&'static str> {
    AI_TOOLS.iter().map(|t| t.value).collect()
}

pub fn get_tool_by_value(value: &str) -> Option<&'static AITool> {
    AI_TOOLS.iter().find(|t| t.value == value)
}

pub fn detect_available_tools(project_path: &std::path::Path) -> Vec<&'static str> {
    AI_TOOLS
        .iter()
        .filter(|tool| project_path.join(tool.skills_dir).exists())
        .map(|tool| tool.value)
        .collect()
}
