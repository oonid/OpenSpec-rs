#[derive(Debug, Clone)]
pub struct AITool {
    pub name: &'static str,
    pub value: &'static str,
    pub skills_dir: &'static str,
    /// Detection paths for the tool. When non-empty, tool is detected if ANY path exists.
    /// Empty slice means "detect by checking if skills_dir exists as a directory".
    pub detection_paths: &'static [&'static str],
}

pub const AI_TOOLS: &[AITool] = &[
    AITool {
        name: "Amazon Q Developer",
        value: "amazon-q",
        skills_dir: ".amazonq",
        detection_paths: &[],
    },
    AITool {
        name: "Antigravity",
        value: "antigravity",
        skills_dir: ".agent",
        detection_paths: &[],
    },
    AITool {
        name: "Auggie (Augment CLI)",
        value: "auggie",
        skills_dir: ".augment",
        detection_paths: &[],
    },
    AITool {
        name: "Bob Shell",
        value: "bob",
        skills_dir: ".bob",
        detection_paths: &[],
    },
    AITool {
        name: "Claude Code",
        value: "claude",
        skills_dir: ".claude",
        detection_paths: &[],
    },
    AITool {
        name: "Cline",
        value: "cline",
        skills_dir: ".cline",
        detection_paths: &[],
    },
    AITool {
        name: "Codex",
        value: "codex",
        skills_dir: ".codex",
        detection_paths: &[],
    },
    AITool {
        name: "ForgeCode",
        value: "forgecode",
        skills_dir: ".forge",
        detection_paths: &[],
    },
    AITool {
        name: "CodeBuddy Code (CLI)",
        value: "codebuddy",
        skills_dir: ".codebuddy",
        detection_paths: &[],
    },
    AITool {
        name: "Continue",
        value: "continue",
        skills_dir: ".continue",
        detection_paths: &[],
    },
    AITool {
        name: "CoStrict",
        value: "costrict",
        skills_dir: ".cospec",
        detection_paths: &[],
    },
    AITool {
        name: "Crush",
        value: "crush",
        skills_dir: ".crush",
        detection_paths: &[],
    },
    AITool {
        name: "Cursor",
        value: "cursor",
        skills_dir: ".cursor",
        detection_paths: &[],
    },
    AITool {
        name: "Factory Droid",
        value: "factory",
        skills_dir: ".factory",
        detection_paths: &[],
    },
    AITool {
        name: "Gemini CLI",
        value: "gemini",
        skills_dir: ".gemini",
        detection_paths: &[],
    },
    AITool {
        name: "GitHub Copilot",
        value: "github-copilot",
        skills_dir: ".github",
        detection_paths: &[
            ".github/copilot-instructions.md",
            ".github/instructions",
            ".github/workflows/copilot-setup-steps.yml",
            ".github/prompts",
            ".github/agents",
            ".github/skills",
            ".github/.mcp.json",
        ],
    },
    AITool {
        name: "iFlow",
        value: "iflow",
        skills_dir: ".iflow",
        detection_paths: &[],
    },
    AITool {
        name: "Junie",
        value: "junie",
        skills_dir: ".junie",
        detection_paths: &[],
    },
    AITool {
        name: "Kilo Code",
        value: "kilocode",
        skills_dir: ".kilocode",
        detection_paths: &[],
    },
    AITool {
        name: "Kimi CLI",
        value: "kimi",
        skills_dir: ".kimi",
        detection_paths: &[],
    },
    AITool {
        name: "Kiro",
        value: "kiro",
        skills_dir: ".kiro",
        detection_paths: &[],
    },
    AITool {
        name: "Lingma",
        value: "lingma",
        skills_dir: ".lingma",
        detection_paths: &[],
    },
    AITool {
        name: "Mistral Vibe",
        value: "vibe",
        skills_dir: ".vibe",
        detection_paths: &[],
    },
    AITool {
        name: "OpenCode",
        value: "opencode",
        skills_dir: ".opencode",
        detection_paths: &[],
    },
    AITool {
        name: "Pi",
        value: "pi",
        skills_dir: ".pi",
        detection_paths: &[],
    },
    AITool {
        name: "Qoder",
        value: "qoder",
        skills_dir: ".qoder",
        detection_paths: &[],
    },
    AITool {
        name: "Qwen Code",
        value: "qwen",
        skills_dir: ".qwen",
        detection_paths: &[],
    },
    AITool {
        name: "RooCode",
        value: "roocode",
        skills_dir: ".roo",
        detection_paths: &[],
    },
    AITool {
        name: "Trae",
        value: "trae",
        skills_dir: ".trae",
        detection_paths: &[],
    },
    AITool {
        name: "Windsurf",
        value: "windsurf",
        skills_dir: ".windsurf",
        detection_paths: &[],
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
        .filter(|tool| {
            if tool.detection_paths.is_empty() {
                // Fall back to checking if skills_dir exists as a directory
                project_path.join(tool.skills_dir).exists()
            } else {
                // Check if ANY of the detection_paths exist
                tool.detection_paths
                    .iter()
                    .any(|path| project_path.join(path).exists())
            }
        })
        .map(|tool| tool.value)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_new_tools_present() {
        let required_tools = vec!["bob", "forgecode", "junie", "kimi", "lingma", "vibe"];
        for tool_value in required_tools {
            assert!(
                AI_TOOLS.iter().any(|t| t.value == tool_value),
                "Tool {} not found in AI_TOOLS",
                tool_value
            );
        }
    }

    #[test]
    fn test_copilot_not_detected_from_bare_github() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create bare .github directory with nothing in it
        fs::create_dir_all(project_path.join(".github")).unwrap();

        let detected = detect_available_tools(project_path);
        assert!(
            !detected.contains(&"github-copilot"),
            "Copilot should not be detected from bare .github/"
        );
    }

    #[test]
    fn test_copilot_detected_from_copilot_prompts() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create .github/prompts directory (one of Copilot's detection paths)
        fs::create_dir_all(project_path.join(".github/prompts")).unwrap();

        let detected = detect_available_tools(project_path);
        assert!(
            detected.contains(&"github-copilot"),
            "Copilot should be detected from .github/prompts"
        );
    }

    #[test]
    fn test_copilot_detected_from_copilot_instructions() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create .github/copilot-instructions.md (one of Copilot's detection paths)
        fs::create_dir_all(project_path.join(".github")).unwrap();
        fs::write(
            project_path.join(".github/copilot-instructions.md"),
            "# Instructions",
        )
        .unwrap();

        let detected = detect_available_tools(project_path);
        assert!(
            detected.contains(&"github-copilot"),
            "Copilot should be detected from .github/copilot-instructions.md"
        );
    }

    #[test]
    fn test_regular_tool_detected_by_skills_dir() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create Claude Code's skills dir
        fs::create_dir_all(project_path.join(".claude")).unwrap();

        let detected = detect_available_tools(project_path);
        assert!(
            detected.contains(&"claude"),
            "Claude Code should be detected from .claude directory"
        );
    }

    #[test]
    fn test_ai_tools_ordered_correctly() {
        let expected_order = vec![
            "amazon-q",
            "antigravity",
            "auggie",
            "bob",
            "claude",
            "cline",
            "codex",
            "forgecode",
            "codebuddy",
            "continue",
            "costrict",
            "crush",
            "cursor",
            "factory",
            "gemini",
            "github-copilot",
            "iflow",
            "junie",
            "kilocode",
            "kimi",
            "kiro",
            "lingma",
            "vibe",
            "opencode",
            "pi",
            "qoder",
            "qwen",
            "roocode",
            "trae",
            "windsurf",
        ];

        let actual_order: Vec<&str> = AI_TOOLS.iter().map(|t| t.value).collect();
        assert_eq!(
            actual_order, expected_order,
            "AI_TOOLS order does not match expected alphabetical order"
        );
    }
}
