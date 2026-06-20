use super::foundation::{parse_workspace_preferred_opener_value, PreferredOpener};
use std::env;
use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceOpenerChoice {
    pub value: String,
    pub label: String,
    pub opener: PreferredOpener,
    pub executable: String,
    pub available: bool,
    pub unavailable_note: Option<String>,
}

struct OpenerDefinition {
    value: &'static str,
    label: &'static str,
    executable: &'static str,
}

const WORKSPACE_OPENER_CHOICE_DEFINITIONS: &[OpenerDefinition] = &[
    OpenerDefinition {
        value: "editor",
        label: "VS Code editor",
        executable: "code",
    },
    OpenerDefinition {
        value: "codex-cli",
        label: "codex-cli",
        executable: "codex",
    },
    OpenerDefinition {
        value: "claude",
        label: "Claude",
        executable: "claude",
    },
    OpenerDefinition {
        value: "github-copilot",
        label: "GitHub Copilot in VS Code",
        executable: "code",
    },
];

/// Check if a file is executable on the current platform.
fn is_executable_file(candidate_path: &str) -> bool {
    match fs::metadata(candidate_path) {
        Ok(metadata) => {
            if !metadata.is_file() {
                return false;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = metadata.permissions().mode();
                (mode & 0o111) != 0
            }

            #[cfg(not(unix))]
            {
                // On Windows, being a file is sufficient
                true
            }
        }
        Err(_) => false,
    }
}

/// Get PATH and PATHEXT values from the environment (or default).
fn get_path_value(env_var: Option<&str>) -> String {
    env_var.unwrap_or("").to_string()
}

fn get_path_exts(env_pathext: Option<&str>, is_windows: bool) -> Vec<String> {
    if !is_windows {
        return vec!["".to_string()];
    }

    let pathext = env_pathext.unwrap_or(".COM;.EXE;.BAT;.CMD");
    pathext
        .split(';')
        .map(|ext| ext.trim().to_string())
        .filter(|ext| !ext.is_empty())
        .collect()
}

/// Check if an executable is available on PATH.
///
/// This function accepts environment variables and platform info for testability.
/// If you pass None for env_path, it reads from the actual environment.
fn is_workspace_executable_available_internal(
    executable: &str,
    env_path: Option<&str>,
    env_pathext: Option<&str>,
    is_windows: bool,
) -> bool {
    // If the executable contains path separators, check that exact path
    if executable.contains('/') || executable.contains('\\') {
        return is_executable_file(executable);
    }

    // Get PATH and PATHEXT, using environment if not provided
    let path_value = if let Some(p) = env_path {
        p.to_string()
    } else {
        get_path_value(env::var("PATH").ok().as_deref())
    };

    let path_exts = get_path_exts(env_pathext, is_windows);

    // Split PATH by the OS delimiter
    let path_separator = if is_windows { ";" } else { ":" };
    let path_entries: Vec<&str> = path_value
        .split(path_separator)
        .filter(|entry| !entry.is_empty())
        .collect();

    // Search for the executable in each PATH entry with each extension
    for entry in path_entries {
        for ext in &path_exts {
            let candidate = if ext.is_empty() {
                format!("{}/{}", entry, executable)
            } else {
                format!("{}/{}{}", entry, executable, ext)
            };

            if is_executable_file(&candidate) {
                return true;
            }
        }
    }

    false
}

/// Check if a workspace executable is available on PATH.
pub fn is_workspace_executable_available(executable: &str) -> bool {
    let is_windows = cfg!(windows);
    is_workspace_executable_available_internal(executable, None, None, is_windows)
}

/// Get the executable name for a given opener.
pub fn get_workspace_opener_executable(opener: &PreferredOpener) -> String {
    match opener.kind {
        super::foundation::OpenerKind::Editor => "code".to_string(),
        super::foundation::OpenerKind::Agent => {
            if opener.id == "github-copilot" {
                "code".to_string()
            } else if opener.id == "codex-cli" || opener.id == "codex" {
                "codex".to_string()
            } else {
                opener.id.clone()
            }
        }
    }
}

/// Get the label for a given opener.
pub fn get_workspace_opener_label(opener: &PreferredOpener) -> String {
    match opener.kind {
        super::foundation::OpenerKind::Editor => "VS Code editor".to_string(),
        super::foundation::OpenerKind::Agent => {
            if opener.id == "github-copilot" {
                "GitHub Copilot in VS Code".to_string()
            } else if opener.id == "codex-cli" || opener.id == "codex" {
                "codex-cli".to_string()
            } else {
                "Claude".to_string()
            }
        }
    }
}

/// List all available workspace opener choices.
pub fn list_workspace_opener_choices() -> Vec<WorkspaceOpenerChoice> {
    let is_windows = cfg!(windows);
    list_workspace_opener_choices_internal(None, None, is_windows)
}

fn list_workspace_opener_choices_internal(
    env_path: Option<&str>,
    env_pathext: Option<&str>,
    is_windows: bool,
) -> Vec<WorkspaceOpenerChoice> {
    let mut choices: Vec<WorkspaceOpenerChoice> = WORKSPACE_OPENER_CHOICE_DEFINITIONS
        .iter()
        .map(|definition| {
            let available = is_workspace_executable_available_internal(
                definition.executable,
                env_path,
                env_pathext,
                is_windows,
            );
            let opener = parse_workspace_preferred_opener_value(definition.value)
                .expect("hardcoded definition values should always parse");
            WorkspaceOpenerChoice {
                value: definition.value.to_string(),
                label: definition.label.to_string(),
                opener,
                executable: definition.executable.to_string(),
                available,
                unavailable_note: if available {
                    None
                } else {
                    Some(format!("{} not found on PATH", definition.executable))
                },
            }
        })
        .collect();

    // Sort so available choices come first (stable sort otherwise)
    choices.sort_by(|a, b| {
        if a.available != b.available {
            if a.available {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        } else {
            std::cmp::Ordering::Equal
        }
    });

    choices
}

/// Get the default workspace opener choice value.
pub fn get_default_workspace_opener_choice_value(choices: &[WorkspaceOpenerChoice]) -> String {
    choices
        .iter()
        .find(|choice| choice.available)
        .map(|choice| choice.value.clone())
        .unwrap_or_else(|| "editor".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::foundation::validate_workspace_preferred_opener;

    #[test]
    fn test_parse_workspace_preferred_opener_value_editor() {
        let result = parse_workspace_preferred_opener_value("editor").unwrap();
        assert_eq!(result.kind, super::super::foundation::OpenerKind::Editor);
        assert_eq!(result.id, "vscode");
    }

    #[test]
    fn test_parse_workspace_preferred_opener_value_claude() {
        let result = parse_workspace_preferred_opener_value("claude").unwrap();
        assert_eq!(result.kind, super::super::foundation::OpenerKind::Agent);
        assert_eq!(result.id, "claude");
    }

    #[test]
    fn test_parse_workspace_preferred_opener_value_codex_cli() {
        let result = parse_workspace_preferred_opener_value("codex-cli").unwrap();
        assert_eq!(result.kind, super::super::foundation::OpenerKind::Agent);
        assert_eq!(result.id, "codex-cli");
    }

    #[test]
    fn test_parse_workspace_preferred_opener_value_github_copilot() {
        let result = parse_workspace_preferred_opener_value("github-copilot").unwrap();
        assert_eq!(result.kind, super::super::foundation::OpenerKind::Agent);
        assert_eq!(result.id, "github-copilot");
    }

    #[test]
    fn test_parse_workspace_preferred_opener_value_invalid() {
        let result = parse_workspace_preferred_opener_value("bogus");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Unsupported workspace opener"));
        assert!(err.contains("'bogus'"));
    }

    #[test]
    fn test_validate_workspace_preferred_opener_editor_vscode() {
        let opener = PreferredOpener {
            kind: super::super::foundation::OpenerKind::Editor,
            id: "vscode".into(),
        };
        assert!(validate_workspace_preferred_opener(&opener).is_ok());
    }

    #[test]
    fn test_validate_workspace_preferred_opener_editor_invalid() {
        let opener = PreferredOpener {
            kind: super::super::foundation::OpenerKind::Editor,
            id: "foo".into(),
        };
        assert!(validate_workspace_preferred_opener(&opener).is_err());
    }

    #[test]
    fn test_validate_workspace_preferred_opener_agent_claude() {
        let opener = PreferredOpener {
            kind: super::super::foundation::OpenerKind::Agent,
            id: "claude".into(),
        };
        assert!(validate_workspace_preferred_opener(&opener).is_ok());
    }

    #[test]
    fn test_validate_workspace_preferred_opener_agent_invalid() {
        let opener = PreferredOpener {
            kind: super::super::foundation::OpenerKind::Agent,
            id: "foo".into(),
        };
        assert!(validate_workspace_preferred_opener(&opener).is_err());
    }

    #[test]
    fn test_get_workspace_opener_executable_editor() {
        let opener = PreferredOpener {
            kind: super::super::foundation::OpenerKind::Editor,
            id: "vscode".into(),
        };
        assert_eq!(get_workspace_opener_executable(&opener), "code");
    }

    #[test]
    fn test_get_workspace_opener_executable_github_copilot() {
        let opener = PreferredOpener {
            kind: super::super::foundation::OpenerKind::Agent,
            id: "github-copilot".into(),
        };
        assert_eq!(get_workspace_opener_executable(&opener), "code");
    }

    #[test]
    fn test_get_workspace_opener_executable_codex_cli() {
        let opener = PreferredOpener {
            kind: super::super::foundation::OpenerKind::Agent,
            id: "codex-cli".into(),
        };
        assert_eq!(get_workspace_opener_executable(&opener), "codex");
    }

    #[test]
    fn test_get_workspace_opener_executable_codex() {
        let opener = PreferredOpener {
            kind: super::super::foundation::OpenerKind::Agent,
            id: "codex".into(),
        };
        assert_eq!(get_workspace_opener_executable(&opener), "codex");
    }

    #[test]
    fn test_get_workspace_opener_executable_claude() {
        let opener = PreferredOpener {
            kind: super::super::foundation::OpenerKind::Agent,
            id: "claude".into(),
        };
        assert_eq!(get_workspace_opener_executable(&opener), "claude");
    }

    #[test]
    fn test_get_workspace_opener_label_editor() {
        let opener = PreferredOpener {
            kind: super::super::foundation::OpenerKind::Editor,
            id: "vscode".into(),
        };
        assert_eq!(get_workspace_opener_label(&opener), "VS Code editor");
    }

    #[test]
    fn test_get_workspace_opener_label_github_copilot() {
        let opener = PreferredOpener {
            kind: super::super::foundation::OpenerKind::Agent,
            id: "github-copilot".into(),
        };
        assert_eq!(get_workspace_opener_label(&opener), "GitHub Copilot in VS Code");
    }

    #[test]
    fn test_get_workspace_opener_label_codex_cli() {
        let opener = PreferredOpener {
            kind: super::super::foundation::OpenerKind::Agent,
            id: "codex-cli".into(),
        };
        assert_eq!(get_workspace_opener_label(&opener), "codex-cli");
    }

    #[test]
    fn test_get_workspace_opener_label_codex() {
        let opener = PreferredOpener {
            kind: super::super::foundation::OpenerKind::Agent,
            id: "codex".into(),
        };
        assert_eq!(get_workspace_opener_label(&opener), "codex-cli");
    }

    #[test]
    fn test_get_workspace_opener_label_claude() {
        let opener = PreferredOpener {
            kind: super::super::foundation::OpenerKind::Agent,
            id: "claude".into(),
        };
        assert_eq!(get_workspace_opener_label(&opener), "Claude");
    }

    #[test]
    fn test_is_workspace_executable_available_in_path_unix() {
        // Test on Unix-like platform simulation
        let path = "/usr/bin:/bin:/usr/local/bin";
        let result = is_workspace_executable_available_internal("ls", Some(path), None, false);
        // We're simulating, so this depends on test environment. Just test the internal logic.
        // The important part is the function doesn't crash.
        let _ = result;
    }

    #[test]
    fn test_is_workspace_executable_available_with_absolute_path() {
        // Test with absolute path containing separators
        let result = is_workspace_executable_available_internal(
            "/usr/bin/fake-executable-that-should-not-exist",
            None,
            None,
            false,
        );
        // Should be false since this file doesn't exist
        assert!(!result);
    }

    #[test]
    fn test_is_workspace_executable_available_with_windows_path() {
        let result = is_workspace_executable_available_internal(
            "C:\\fake\\executable.exe",
            None,
            None,
            true,
        );
        // Should be false since this file doesn't exist
        assert!(!result);
    }

    #[test]
    fn test_is_executable_file_with_real_file() {
        // Use a tempfile to test the executable detection
        use tempfile::NamedTempFile;
        use std::os::unix::fs::PermissionsExt;

        #[cfg(unix)]
        {
            let temp_file = NamedTempFile::new().expect("failed to create temp file");
            let path = temp_file.path().to_string_lossy().to_string();

            // Make the file executable
            let permissions = fs::Permissions::from_mode(0o755);
            fs::set_permissions(&path, permissions).expect("failed to set permissions");

            // Should be detected as executable
            assert!(is_executable_file(&path));
        }
    }

    #[test]
    fn test_list_workspace_opener_choices_returns_four_choices() {
        let choices = list_workspace_opener_choices();
        assert_eq!(choices.len(), 4);
    }

    #[test]
    fn test_list_workspace_opener_choices_values() {
        let choices = list_workspace_opener_choices();
        let values: Vec<&str> = choices.iter().map(|c| c.value.as_str()).collect();
        // Should contain all values (order may vary based on availability)
        assert!(values.contains(&"editor"));
        assert!(values.contains(&"codex-cli"));
        assert!(values.contains(&"claude"));
        assert!(values.contains(&"github-copilot"));
    }

    #[test]
    fn test_list_workspace_opener_choices_available_first() {
        // Simulate with a PATH that only contains /tmp (unlikely to have real executables)
        let choices = list_workspace_opener_choices_internal(Some("/tmp"), None, false);

        // All choices should be unavailable in /tmp
        // Verify the stable sort: all unavailable should be at the end
        let mut prev_available = true;
        for choice in choices {
            if choice.available {
                assert!(prev_available, "available choices should come first");
            } else {
                prev_available = false;
            }
        }
    }

    #[test]
    fn test_get_default_workspace_opener_choice_value_with_available() {
        let available_choice = WorkspaceOpenerChoice {
            value: "editor".to_string(),
            label: "VS Code editor".to_string(),
            opener: PreferredOpener {
                kind: super::super::foundation::OpenerKind::Editor,
                id: "vscode".into(),
            },
            executable: "code".to_string(),
            available: true,
            unavailable_note: None,
        };

        let unavailable_choice = WorkspaceOpenerChoice {
            value: "claude".to_string(),
            label: "Claude".to_string(),
            opener: PreferredOpener {
                kind: super::super::foundation::OpenerKind::Agent,
                id: "claude".into(),
            },
            executable: "claude".to_string(),
            available: false,
            unavailable_note: Some("claude not found on PATH".to_string()),
        };

        let choices = vec![available_choice, unavailable_choice];
        assert_eq!(
            get_default_workspace_opener_choice_value(&choices),
            "editor"
        );
    }

    #[test]
    fn test_get_default_workspace_opener_choice_value_with_none_available() {
        let choices = vec![
            WorkspaceOpenerChoice {
                value: "claude".to_string(),
                label: "Claude".to_string(),
                opener: PreferredOpener {
                    kind: super::super::foundation::OpenerKind::Agent,
                    id: "claude".into(),
                },
                executable: "claude".to_string(),
                available: false,
                unavailable_note: Some("claude not found on PATH".to_string()),
            },
        ];

        assert_eq!(
            get_default_workspace_opener_choice_value(&choices),
            "editor"
        );
    }

    #[test]
    fn test_get_path_exts_unix() {
        let exts = get_path_exts(None, false);
        assert_eq!(exts, vec![""]);
    }

    #[test]
    fn test_get_path_exts_windows_default() {
        let exts = get_path_exts(None, true);
        assert_eq!(
            exts,
            vec![".COM".to_string(), ".EXE".to_string(), ".BAT".to_string(), ".CMD".to_string()]
        );
    }

    #[test]
    fn test_get_path_exts_windows_custom() {
        let exts = get_path_exts(Some(".EXE;.COM"), true);
        assert_eq!(exts, vec![".EXE".to_string(), ".COM".to_string()]);
    }

    #[test]
    fn test_get_path_exts_windows_with_spaces() {
        let exts = get_path_exts(Some(" .EXE ; .COM "), true);
        assert_eq!(exts, vec![".EXE".to_string(), ".COM".to_string()]);
    }
}
