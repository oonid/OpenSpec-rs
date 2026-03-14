use std::io::{self, Write};
use std::path::Path;

use crate::ai_tools::generator::{get_tool_by_value, AI_TOOLS};
use crate::core::config::ProjectConfig;
use crate::core::error::{OpenSpecError, Result};
use crate::templates::{
    generate_commands, generate_skill_content, get_adapter, get_command_contents,
    get_skill_templates,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_SCHEMA: &str = "spec-driven";

pub fn validate_write_permissions(path: &Path) -> Result<()> {
    let test_file = path.join(".openspec_write_test");
    std::fs::write(&test_file, "").map_err(|e| {
        OpenSpecError::Custom(format!(
            "Insufficient permissions to write to {}: {}",
            path.display(),
            e
        ))
    })?;
    let _ = std::fs::remove_file(&test_file);
    Ok(())
}

fn detect_available_tools(project_path: &Path) -> Vec<&'static str> {
    AI_TOOLS
        .iter()
        .filter(|tool| project_path.join(tool.skills_dir).exists())
        .map(|tool| tool.value)
        .collect()
}

pub fn resolve_tool_selection(
    tools_arg: &Option<String>,
    project_path: &Path,
    _extend_mode: bool,
) -> Result<Vec<String>> {
    if let Some(ref arg) = tools_arg {
        return parse_tools_arg(arg);
    }

    let detected = detect_available_tools(project_path);

    if !is_interactive() {
        if detected.is_empty() {
            return Err(OpenSpecError::Custom(
                "No tools detected and no --tools flag provided. Use --tools all, --tools none, or --tools opencode,claude,...".to_string()
            ));
        }
        return Ok(detected.into_iter().map(|s| s.to_string()).collect());
    }

    interactive_tool_selection(detected)
}

fn is_interactive() -> bool {
    atty::is(atty::Stream::Stdout) && atty::is(atty::Stream::Stdin)
}

fn parse_tools_arg(arg: &str) -> Result<Vec<String>> {
    let arg = arg.trim().to_lowercase();

    if arg == "all" {
        return Ok(AI_TOOLS.iter().map(|t| t.value.to_string()).collect());
    }

    if arg == "none" {
        return Ok(vec![]);
    }

    let tools: Vec<String> = arg
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if tools.is_empty() {
        return Err(OpenSpecError::Custom(
            "The --tools option requires at least one tool ID when not using 'all' or 'none'."
                .to_string(),
        ));
    }

    for tool in &tools {
        if tool == "all" || tool == "none" {
            return Err(OpenSpecError::Custom(
                "Cannot combine reserved values 'all' or 'none' with specific tool IDs."
                    .to_string(),
            ));
        }
        if get_tool_by_value(tool).is_none() {
            let valid_tools: Vec<&str> = AI_TOOLS.iter().map(|t| t.value).collect();
            return Err(OpenSpecError::Custom(format!(
                "Invalid tool '{}'. Available tools: {}",
                tool,
                valid_tools.join(", ")
            )));
        }
    }

    let mut seen = std::collections::HashSet::new();
    Ok(tools
        .into_iter()
        .filter(|t| seen.insert(t.clone()))
        .collect())
}

fn interactive_tool_selection(detected: Vec<&'static str>) -> Result<Vec<String>> {
    println!("\nSelect AI tools to configure:\n");

    for (i, tool) in AI_TOOLS.iter().enumerate() {
        let detected_marker = if detected.contains(&tool.value) {
            " [detected]"
        } else {
            ""
        };
        println!("  {:2}. {}{}", i + 1, tool.name, detected_marker);
    }

    print!("\nEnter tool numbers (comma-separated), 'all', or 'none': ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to read input: {}", e)))?;

    let input = input.trim().to_lowercase();

    if input == "all" {
        return Ok(AI_TOOLS.iter().map(|t| t.value.to_string()).collect());
    }
    if input == "none" {
        return Ok(vec![]);
    }

    let mut selected = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Ok(num) = part.parse::<usize>() {
            if num >= 1 && num <= AI_TOOLS.len() {
                selected.push(AI_TOOLS[num - 1].value.to_string());
            }
        }
    }

    if selected.is_empty() {
        return Err(OpenSpecError::Custom(
            "No valid tools selected.".to_string(),
        ));
    }

    Ok(selected)
}

pub fn create_directory_structure(openspec_path: &Path, extend_mode: bool) -> Result<()> {
    let directories = vec![
        openspec_path.to_path_buf(),
        openspec_path.join("specs"),
        openspec_path.join("changes"),
        openspec_path.join("changes").join("archive"),
    ];

    for dir in &directories {
        std::fs::create_dir_all(dir).map_err(|e| {
            OpenSpecError::Custom(format!(
                "Failed to create directory {}: {}",
                dir.display(),
                e
            ))
        })?;
    }

    if !extend_mode {
        println!("OpenSpec structure created");
    }
    Ok(())
}

pub fn generate_skills_for_tools(project_path: &Path, tools: &[String]) -> Result<()> {
    if tools.is_empty() {
        return Ok(());
    }

    let skill_entries = get_skill_templates(None);
    let command_contents = get_command_contents(None);

    for tool_id in tools {
        let tool = match get_tool_by_value(tool_id) {
            Some(t) => t,
            None => continue,
        };

        print!("Setting up {}...", tool.name);

        let skills_dir = project_path.join(tool.skills_dir).join("skills");
        std::fs::create_dir_all(&skills_dir).map_err(|e| {
            OpenSpecError::Custom(format!("Failed to create skills directory: {}", e))
        })?;

        for entry in &skill_entries {
            let skill_dir = skills_dir.join(entry.dir_name);
            let skill_file = skill_dir.join("SKILL.md");

            std::fs::create_dir_all(&skill_dir).map_err(|e| {
                OpenSpecError::Custom(format!("Failed to create skill directory: {}", e))
            })?;

            let content = generate_skill_content(entry.template, VERSION);
            std::fs::write(&skill_file, content)
                .map_err(|e| OpenSpecError::Custom(format!("Failed to write skill file: {}", e)))?;
        }

        if let Some(adapter) = get_adapter(tool_id) {
            let generated_commands = generate_commands(&command_contents, adapter.as_ref());

            for cmd in generated_commands {
                let cmd_path = project_path.join(&cmd.path);
                if let Some(parent) = cmd_path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        OpenSpecError::Custom(format!("Failed to create commands directory: {}", e))
                    })?;
                }
                std::fs::write(&cmd_path, &cmd.file_content).map_err(|e| {
                    OpenSpecError::Custom(format!("Failed to write command file: {}", e))
                })?;
            }
        }

        println!(
            " {} skills, {} commands configured",
            skill_entries.len(),
            command_contents.len()
        );
    }

    Ok(())
}
pub fn create_config(
    openspec_path: &Path,
    _extend_mode: bool,
    force: bool,
) -> Result<&'static str> {
    let config_path = openspec_path.join("config.yaml");

    if config_path.exists() {
        return Ok("exists");
    }

    if !is_interactive() && !force {
        return Ok("skipped");
    }

    let config = ProjectConfig::default();
    let yaml_content = serde_yaml::to_string(&config)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to serialize config: {}", e)))?;

    std::fs::write(&config_path, yaml_content)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to write config: {}", e)))?;

    Ok("created")
}

pub fn display_success_message(_project_path: &Path, tools: &[String], config_status: &str) {
    use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};

    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    let _ = stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Green)));
    let _ = writeln!(stdout);
    let _ = stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Rgb(78, 143, 242))));
    let _ = writeln!(stdout, "OpenSpec Setup Complete");
    let _ = stdout.reset();

    println!();

    if !tools.is_empty() {
        let tool_names: Vec<&str> = tools
            .iter()
            .filter_map(|t| get_tool_by_value(t).map(|tool| tool.name))
            .collect();
        println!("Configured: {}", tool_names.join(", "));

        let skill_count = get_skill_templates(None).len();
        if skill_count > 0 {
            println!("{} skills generated", skill_count);
        }
    }

    match config_status {
        "created" => println!("Config: openspec/config.yaml (schema: {})", DEFAULT_SCHEMA),
        "exists" => println!("Config: openspec/config.yaml (exists)"),
        "skipped" => println!("Config: skipped (non-interactive mode)"),
        _ => {}
    }

    println!();
    println!("Getting started:");
    println!("  Start your first change: /opsx:propose \"your idea\"");
    println!();
    println!("Learn more: https://github.com/Fission-AI/OpenSpec");
    println!();

    if !tools.is_empty() {
        println!("Restart your IDE for slash commands to take effect.");
    }

    println!();
}

pub fn run_init(
    path: &str,
    tools: Option<&str>,
    force: bool,
    _profile: Option<&str>,
) -> Result<()> {
    let project_path = std::path::Path::new(path)
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(path));
    let openspec_path = project_path.join("openspec");

    let extend_mode = openspec_path.exists();

    if !extend_mode {
        validate_write_permissions(&project_path)?;
    }

    let tools_opt = tools.map(|s| s.to_string());
    let selected_tools = resolve_tool_selection(&tools_opt, &project_path, extend_mode)?;

    create_directory_structure(&openspec_path, extend_mode)?;

    generate_skills_for_tools(&project_path, &selected_tools)?;

    let config_status = create_config(&openspec_path, extend_mode, force)?;

    display_success_message(&project_path, &selected_tools, config_status);

    Ok(())
}
