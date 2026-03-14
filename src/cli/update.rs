use crate::core::config::{ConfigManager, OPENSPEC_DIR_NAME};
use crate::core::error::{OpenSpecError, Result};

pub fn run_update(force: bool) -> Result<()> {
    let project_root = std::env::current_dir()
        .map_err(|e| OpenSpecError::Custom(format!("Failed to get current directory: {}", e)))?;

    let openspec_path = project_root.join(OPENSPEC_DIR_NAME);

    if !openspec_path.exists() || !openspec_path.is_dir() {
        return Err(OpenSpecError::Custom(
            "No OpenSpec directory found. Run 'openspec init' first.".to_string(),
        ));
    }

    let global_config = ConfigManager::load_global_config();
    let profile = global_config.profile;
    let delivery = global_config.delivery;
    let workflows = global_config.workflows;

    println!("Profile: {}", profile);
    println!("Delivery: {}", delivery);
    println!("Workflows: {}", workflows.join(", "));
    println!();

    let tools = detect_configured_tools(&project_root);

    if tools.is_empty() {
        println!("No configured tools found.");
        println!("Run 'openspec init' to set up tools.");
        return Ok(());
    }

    println!("Configured tools: {}", tools.join(", "));

    if force {
        println!("\nForce updating {} tool(s)...", tools.len());
    } else {
        println!("\nTools are up to date.");
        println!("Use --force to refresh files anyway.");
        return Ok(());
    }

    let mut updated = Vec::new();
    let mut failed = Vec::new();

    for tool in &tools {
        match update_tool(&project_root, tool, &workflows, &delivery) {
            Ok(()) => updated.push(tool.clone()),
            Err(e) => failed.push((tool.clone(), e.to_string())),
        }
    }

    println!();
    if !updated.is_empty() {
        println!("✓ Updated: {}", updated.join(", "));
    }
    if !failed.is_empty() {
        for (tool, error) in &failed {
            println!("✗ Failed: {} ({})", tool, error);
        }
    }

    println!();
    println!("Restart your IDE for changes to take effect.");

    Ok(())
}

fn detect_configured_tools(project_root: &std::path::Path) -> Vec<String> {
    let mut tools = Vec::new();

    let opencode_dir = project_root.join(".opencode");
    if opencode_dir.exists() {
        tools.push("opencode".to_string());
    }

    let claude_dir = project_root.join(".claude");
    if claude_dir.exists() {
        tools.push("claude".to_string());
    }

    let cursor_dir = project_root.join(".cursor");
    if cursor_dir.exists() {
        tools.push("cursor".to_string());
    }

    let copilot_dir = project_root.join(".github").join("copilot");
    if copilot_dir.exists() {
        tools.push("copilot".to_string());
    }

    let windsurf_dir = project_root.join(".windsurf");
    if windsurf_dir.exists() {
        tools.push("windsurf".to_string());
    }

    let aider_dir = project_root.join(".aider");
    if aider_dir.exists() {
        tools.push("aider".to_string());
    }

    tools.sort();
    tools
}

fn update_tool(
    project_root: &std::path::Path,
    tool: &str,
    workflows: &[String],
    _delivery: &str,
) -> Result<()> {
    let skills_dir = match tool {
        "opencode" => project_root.join(".opencode").join("commands"),
        "claude" => project_root.join(".claude").join("commands"),
        "cursor" => project_root.join(".cursor").join("commands"),
        "copilot" => project_root
            .join(".github")
            .join("copilot")
            .join("commands"),
        "windsurf" => project_root.join(".windsurf").join("commands"),
        "aider" => project_root.join(".aider").join("commands"),
        _ => return Err(OpenSpecError::Custom(format!("Unknown tool: {}", tool))),
    };

    std::fs::create_dir_all(&skills_dir)
        .map_err(|e| OpenSpecError::Custom(format!("Failed to create skills directory: {}", e)))?;

    for workflow in workflows {
        let skill_content = generate_skill_content(workflow);
        let skill_file = skills_dir.join(format!("{}.md", workflow));

        std::fs::write(&skill_file, skill_content)
            .map_err(|e| OpenSpecError::Custom(format!("Failed to write skill file: {}", e)))?;
    }

    Ok(())
}

fn generate_skill_content(workflow: &str) -> String {
    match workflow {
        "propose" => include_str!("../templates/skills/propose.md").to_string(),
        "explore" => include_str!("../templates/skills/explore.md").to_string(),
        "apply" => include_str!("../templates/skills/apply-change.md").to_string(),
        "archive" => include_str!("../templates/skills/archive-change.md").to_string(),
        _ => format!(
            "# {} Workflow\n\nThis workflow is not yet documented.\n",
            workflow
        ),
    }
}
