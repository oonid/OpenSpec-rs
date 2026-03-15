use std::path::PathBuf;
use tempfile::TempDir;

fn get_binary_path() -> PathBuf {
    if let Ok(path) = std::env::var("OPENSPEC_BINARY") {
        return PathBuf::from(path);
    }

    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path.pop();
    path.join("openspec")
}

fn run_openspec(args: &[&str], cwd: &PathBuf) -> Result<String, String> {
    let binary = get_binary_path();
    let output = std::process::Command::new(&binary)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("Failed to execute openspec: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[test]
fn test_init_creates_directory_structure() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    let result = run_openspec(&["init", "--tools", "none", "."], &path);
    assert!(result.is_ok(), "init failed: {:?}", result);

    assert!(
        path.join("openspec").exists(),
        "openspec directory not created"
    );
    assert!(
        path.join("openspec/specs").exists(),
        "specs directory not created"
    );
    assert!(
        path.join("openspec/changes").exists(),
        "changes directory not created"
    );
    assert!(
        path.join("openspec/changes/archive").exists(),
        "archive directory not created"
    );
}

#[test]
fn test_init_with_tools_creates_skill_directories() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    let result = run_openspec(&["init", "--tools", "opencode", "."], &path);
    assert!(result.is_ok(), "init failed: {:?}", result);

    assert!(
        path.join(".opencode").exists(),
        ".opencode directory not created"
    );
    assert!(
        path.join(".opencode/skills").exists(),
        "skills directory not created"
    );
}

#[test]
fn test_new_change_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    run_openspec(&["init", "--tools", "none", "."], &path).unwrap();
    let result = run_openspec(
        &["new", "change", "test-feature", "--description", "Test"],
        &path,
    );
    assert!(result.is_ok(), "new change failed: {:?}", result);

    assert!(
        path.join("openspec/changes/test-feature").exists(),
        "change directory not created"
    );
    assert!(
        path.join("openspec/changes/test-feature/.openspec.yaml")
            .exists(),
        ".openspec.yaml not created"
    );
}

#[test]
fn test_status_shows_change_status() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    run_openspec(&["init", "--tools", "none", "."], &path).unwrap();
    run_openspec(
        &["new", "change", "test-feature", "--description", "Test"],
        &path,
    )
    .unwrap();

    let result = run_openspec(&["status", "--change", "test-feature"], &path);
    assert!(result.is_ok(), "status failed: {:?}", result);
}

#[test]
fn test_list_shows_changes() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    run_openspec(&["init", "--tools", "none", "."], &path).unwrap();
    run_openspec(
        &["new", "change", "test-feature", "--description", "Test"],
        &path,
    )
    .unwrap();

    let result = run_openspec(&["list"], &path);
    assert!(result.is_ok(), "list failed: {:?}", result);
    let output = result.unwrap();
    assert!(output.contains("test-feature"), "change not listed");
}

#[test]
fn test_schemas_lists_available_schemas() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    run_openspec(&["init", "--tools", "none", "."], &path).unwrap();

    let result = run_openspec(&["schemas"], &path);
    assert!(result.is_ok(), "schemas failed: {:?}", result);
    let output = result.unwrap();
    assert!(
        output.contains("spec-driven"),
        "spec-driven schema not listed"
    );
}

#[test]
fn test_validate_empty_change() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    run_openspec(&["init", "--tools", "none", "."], &path).unwrap();
    run_openspec(
        &["new", "change", "test-feature", "--description", "Test"],
        &path,
    )
    .unwrap();

    let result = run_openspec(&["validate", "test-feature"], &path);
    assert!(
        result.is_err() || result.unwrap().contains("issues"),
        "validation should report issues for empty change"
    );
}

#[test]
fn test_config_list() {
    let result = run_openspec(&["config", "--list"], &std::env::current_dir().unwrap());
    assert!(result.is_ok(), "config list failed: {:?}", result);
}

#[test]
fn test_version_flag() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    let result = run_openspec(&["--version"], &path);
    assert!(result.is_ok(), "version flag failed: {:?}", result);
    let output = result.unwrap();
    assert!(output.contains("openspec"), "version output missing name");
}

#[test]
fn test_help_flag() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    let result = run_openspec(&["--help"], &path);
    assert!(result.is_ok(), "help flag failed: {:?}", result);
    let output = result.unwrap();
    assert!(output.contains("init"), "help missing init command");
    assert!(output.contains("status"), "help missing status command");
    assert!(output.contains("list"), "help missing list command");
}

#[test]
fn test_show_command() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    run_openspec(&["init", "--tools", "none", "."], &path).unwrap();
    run_openspec(
        &["new", "change", "test-feature", "--description", "Test"],
        &path,
    )
    .unwrap();

    let result = run_openspec(&["show", "test-feature"], &path);
    assert!(result.is_ok(), "show failed: {:?}", result);
}

#[test]
fn test_show_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    run_openspec(&["init", "--tools", "none", "."], &path).unwrap();
    run_openspec(
        &["new", "change", "test-feature", "--description", "Test"],
        &path,
    )
    .unwrap();

    let result = run_openspec(&["show", "test-feature", "--json"], &path);
    assert!(result.is_ok(), "show --json failed: {:?}", result);
    let output = result.unwrap();
    assert!(
        output.contains("test-feature"),
        "JSON output missing change name"
    );
}

#[test]
fn test_status_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    run_openspec(&["init", "--tools", "none", "."], &path).unwrap();
    run_openspec(
        &["new", "change", "test-feature", "--description", "Test"],
        &path,
    )
    .unwrap();

    let result = run_openspec(&["status", "--change", "test-feature", "--json"], &path);
    assert!(result.is_ok(), "status --json failed: {:?}", result);
    let output = result.unwrap();
    assert!(
        output.contains("artifacts"),
        "JSON output missing artifacts"
    );
}

#[test]
fn test_list_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    run_openspec(&["init", "--tools", "none", "."], &path).unwrap();
    run_openspec(
        &["new", "change", "test-feature", "--description", "Test"],
        &path,
    )
    .unwrap();

    let result = run_openspec(&["list", "--json"], &path);
    assert!(result.is_ok(), "list --json failed: {:?}", result);
    let output = result.unwrap();
    assert!(
        output.contains("test-feature"),
        "JSON output missing change"
    );
    assert!(
        output.contains("changes"),
        "JSON output missing changes array"
    );
}

#[test]
fn test_schemas_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    run_openspec(&["init", "--tools", "none", "."], &path).unwrap();

    let result = run_openspec(&["schemas", "--json"], &path);
    assert!(result.is_ok(), "schemas --json failed: {:?}", result);
    let output = result.unwrap();
    assert!(output.contains("spec-driven"), "JSON output missing schema");
    assert!(
        output.contains("schemas"),
        "JSON output missing schemas array"
    );
}

#[test]
fn test_completion_generate() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    run_openspec(&["init", "--tools", "none", "."], &path).unwrap();

    let result = run_openspec(&["completion", "generate", "bash"], &path);
    assert!(
        result.is_ok(),
        "completion generate bash failed: {:?}",
        result
    );
    let output = result.unwrap();
    assert!(
        output.contains("_openspec"),
        "bash completion missing function"
    );
}

#[test]
fn test_complete_changes() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    run_openspec(&["init", "--tools", "none", "."], &path).unwrap();
    run_openspec(
        &["new", "change", "test-feature", "--description", "Test"],
        &path,
    )
    .unwrap();

    let result = run_openspec(&["__complete", "changes"], &path);
    assert!(result.is_ok(), "__complete changes failed: {:?}", result);
    let output = result.unwrap();
    assert!(
        output.contains("test-feature"),
        "completion output missing change"
    );
}
