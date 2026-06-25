use serde_json::Value;
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
        .env_remove("NO_COLOR")
        .output()
        .map_err(|e| format!("Failed to execute openspec: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn run_openspec_with_clean_env(args: &[&str], cwd: &PathBuf) -> Result<String, String> {
    let binary = get_binary_path();
    let env_root = TempDir::new().unwrap();
    let xdg_config_home = env_root.path().join("config-home");
    let xdg_data_home = env_root.path().join("data-home");
    std::fs::create_dir_all(&xdg_config_home).unwrap();
    std::fs::create_dir_all(&xdg_data_home).unwrap();

    let output = std::process::Command::new(&binary)
        .args(args)
        .current_dir(cwd)
        .env_remove("NO_COLOR")
        .env("HOME", env_root.path())
        .env("XDG_CONFIG_HOME", &xdg_config_home)
        .env("XDG_DATA_HOME", &xdg_data_home)
        .output()
        .map_err(|e| format!("Failed to execute openspec: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[derive(Debug)]
struct CapturedCommandOutput {
    stdout: String,
    stderr: String,
    success: bool,
}

fn run_openspec_capture(args: &[&str], cwd: &PathBuf) -> CapturedCommandOutput {
    let binary = get_binary_path();
    let output = std::process::Command::new(&binary)
        .args(args)
        .current_dir(cwd)
        .env_remove("NO_COLOR")
        .output()
        .unwrap();

    CapturedCommandOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        success: output.status.success(),
    }
}

fn run_openspec_capture_in_env_root(
    args: &[&str],
    cwd: &PathBuf,
    env_root: &std::path::Path,
    extra_env: &[(&str, &str)],
) -> CapturedCommandOutput {
    let binary = get_binary_path();
    let xdg_config_home = env_root.join("config-home");
    let xdg_data_home = env_root.join("data-home");
    std::fs::create_dir_all(&xdg_config_home).unwrap();
    std::fs::create_dir_all(&xdg_data_home).unwrap();

    let mut command = std::process::Command::new(&binary);
    command
        .args(args)
        .current_dir(cwd)
        .env_remove("NO_COLOR")
        .env("HOME", env_root)
        .env("XDG_CONFIG_HOME", &xdg_config_home)
        .env("XDG_DATA_HOME", &xdg_data_home);

    for (key, value) in extra_env {
        command.env(key, value);
    }

    let output = command.output().unwrap();

    CapturedCommandOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        success: output.status.success(),
    }
}

fn run_openspec_capture_with_clean_env(
    args: &[&str],
    cwd: &PathBuf,
    extra_env: &[(&str, &str)],
) -> CapturedCommandOutput {
    let env_root = TempDir::new().unwrap();
    run_openspec_capture_in_env_root(args, cwd, env_root.path(), extra_env)
}

fn assert_embedded_schema_commands_work(cwd: &PathBuf) {
    let schemas_result = run_openspec_with_clean_env(&["schemas"], cwd);
    assert!(
        schemas_result.is_ok(),
        "schemas failed: {:?}",
        schemas_result
    );
    let schemas_output = schemas_result.unwrap();
    assert!(
        schemas_output.contains("spec-driven"),
        "spec-driven schema not listed"
    );

    let schemas_json_result = run_openspec_with_clean_env(&["schemas", "--json"], cwd);
    assert!(
        schemas_json_result.is_ok(),
        "schemas --json failed: {:?}",
        schemas_json_result
    );
    let schemas_json: Value = serde_json::from_str(&schemas_json_result.unwrap()).unwrap();
    let spec_driven = schemas_json["schemas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|schema| schema["name"] == "spec-driven")
        .unwrap();
    assert_eq!(spec_driven["source"], "package");

    let which_result =
        run_openspec_with_clean_env(&["schema", "which", "spec-driven", "--json"], cwd);
    assert!(
        which_result.is_ok(),
        "schema which failed: {:?}",
        which_result
    );
    let which_json: Value = serde_json::from_str(&which_result.unwrap()).unwrap();
    assert_eq!(which_json["name"], "spec-driven");
    assert_eq!(which_json["source"], "package");
    assert_eq!(which_json["path"], "embedded:spec-driven.yaml");
    assert_eq!(which_json["shadows"].as_array().unwrap().len(), 0);

    let templates_result = run_openspec_with_clean_env(&["templates", "--json"], cwd);
    assert!(
        templates_result.is_ok(),
        "templates --json failed: {:?}",
        templates_result
    );
    let templates_json: Value = serde_json::from_str(&templates_result.unwrap()).unwrap();
    assert_eq!(templates_json["proposal"], "proposal.md");
    assert_eq!(templates_json["specs"], "spec.md");
    assert_eq!(templates_json["design"], "design.md");
    assert_eq!(templates_json["tasks"], "tasks.md");

    let validate_result =
        run_openspec_with_clean_env(&["schema", "validate", "spec-driven", "--json"], cwd);
    assert!(
        validate_result.is_ok(),
        "schema validate failed: {:?}",
        validate_result
    );
    let validate_json: Value = serde_json::from_str(&validate_result.unwrap()).unwrap();
    assert_eq!(validate_json["name"], "spec-driven");
    assert_eq!(validate_json["valid"], true);
    assert_eq!(validate_json["issues"].as_array().unwrap().len(), 0);
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
fn test_new_change_json_output_and_initiative_link() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();
    let store_path = path.join("team-context");
    let store_path_str = store_path.to_string_lossy().to_string();

    run_openspec(&["init", "--tools", "none", "."], &path).unwrap();
    let create_initiative = run_openspec(
        &[
            "initiative",
            "create",
            "roadmap",
            "--title",
            "Roadmap",
            "--summary",
            "Shared planning context",
            "--store-path",
            &store_path_str,
        ],
        &path,
    );
    assert!(
        create_initiative.is_ok(),
        "initiative create failed: {:?}",
        create_initiative
    );

    let result = run_openspec(
        &[
            "new",
            "change",
            "test-feature",
            "--initiative",
            "roadmap",
            "--store-path",
            &store_path_str,
            "--json",
        ],
        &path,
    );
    assert!(result.is_ok(), "new change --json failed: {:?}", result);

    let output = result.unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["change"]["id"], "test-feature");
    assert_eq!(value["change"]["path"], "openspec/changes/test-feature");
    assert_eq!(
        value["change"]["metadataPath"],
        "openspec/changes/test-feature/.openspec.yaml"
    );
    // Linked change → initiative present in the output.
    assert_eq!(value["initiative"]["store"], "team-context");
    assert_eq!(value["initiative"]["id"], "roadmap");

    let metadata =
        std::fs::read_to_string(path.join("openspec/changes/test-feature/.openspec.yaml")).unwrap();
    assert!(metadata.contains("initiative:"));
    assert!(metadata.contains("store: team-context"));
    assert!(metadata.contains("id: roadmap"));
}

#[test]
fn test_new_change_json_output_writes_goal_and_affected_areas() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    run_openspec(&["init", "--tools", "none", "."], &path).unwrap();

    let result = run_openspec(
        &[
            "new",
            "change",
            "add-telemetry",
            "--description",
            "Add telemetry",
            "--goal",
            "Ship telemetry",
            "--affected-areas",
            "cli, docs , ,",
            "--json",
        ],
        &path,
    );
    assert!(result.is_ok(), "new change --json failed: {:?}", result);

    let output = result.unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["change"]["id"], "add-telemetry");
    assert_eq!(value["change"]["path"], "openspec/changes/add-telemetry");
    assert!(value.get("initiative").is_none());

    let metadata_path = path
        .join("openspec")
        .join("changes")
        .join("add-telemetry")
        .join(".openspec.yaml");
    let metadata: serde_yaml::Value =
        serde_yaml::from_str(&std::fs::read_to_string(metadata_path).unwrap()).unwrap();
    assert_eq!(metadata["schema"], "spec-driven");
    assert_eq!(metadata["goal"], "Ship telemetry");
    assert_eq!(
        metadata["affected_areas"],
        serde_yaml::to_value(vec!["cli", "docs"]).unwrap()
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
    let result = run_openspec(&["config", "list"], &std::env::current_dir().unwrap());
    assert!(result.is_ok(), "config list failed: {:?}", result);
}

#[test]
fn test_config_subcommands_roundtrip_with_clean_env() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();
    let env_root = TempDir::new().unwrap();

    let set_profile = run_openspec_capture_in_env_root(
        &["config", "set", "profile", "custom"],
        &path,
        env_root.path(),
        &[],
    );
    assert!(
        set_profile.success,
        "config set profile failed: {:?}",
        set_profile
    );

    let set_feature_flag = run_openspec_capture_in_env_root(
        &["config", "set", "featureFlags.beta", "true"],
        &path,
        env_root.path(),
        &[],
    );
    assert!(
        set_feature_flag.success,
        "config set feature flag failed: {:?}",
        set_feature_flag
    );

    let profile_output =
        run_openspec_capture_in_env_root(&["config", "profile"], &path, env_root.path(), &[]);
    assert!(
        profile_output.success,
        "config profile failed: {:?}",
        profile_output
    );
    assert_eq!(profile_output.stdout.trim(), "custom");

    let profile_core = run_openspec_capture_in_env_root(
        &["config", "profile", "core"],
        &path,
        env_root.path(),
        &[],
    );
    assert!(
        profile_core.success,
        "config profile core failed: {:?}",
        profile_core
    );
    assert_eq!(profile_core.stdout.trim(), "core");

    let list_output = run_openspec_capture_in_env_root(
        &["config", "list", "--json"],
        &path,
        env_root.path(),
        &[],
    );
    assert!(
        list_output.success,
        "config list --json failed: {:?}",
        list_output
    );
    let list_json: Value = serde_json::from_str(&list_output.stdout).unwrap();
    assert_eq!(list_json["profile"], "core");
    assert_eq!(list_json["featureFlags"]["beta"], true);

    let unset_feature_flag = run_openspec_capture_in_env_root(
        &["config", "unset", "featureFlags.beta"],
        &path,
        env_root.path(),
        &[],
    );
    assert!(
        unset_feature_flag.success,
        "config unset feature flag failed: {:?}",
        unset_feature_flag
    );

    let get_missing_flag = run_openspec_capture_in_env_root(
        &["config", "get", "featureFlags.beta"],
        &path,
        env_root.path(),
        &[],
    );
    assert!(
        !get_missing_flag.success,
        "config get missing key should fail"
    );
    assert!(get_missing_flag.stderr.contains("not found"));

    // `reset` requires --all (mirrors upstream); bare reset must be refused.
    let reset_no_flag =
        run_openspec_capture_in_env_root(&["config", "reset"], &path, env_root.path(), &[]);
    assert!(
        !reset_no_flag.success,
        "config reset without --all should fail"
    );
    assert!(reset_no_flag.stderr.contains("--all"));

    let reset_output = run_openspec_capture_in_env_root(
        &["config", "reset", "--all"],
        &path,
        env_root.path(),
        &[],
    );
    assert!(
        reset_output.success,
        "config reset --all failed: {:?}",
        reset_output
    );

    let after_reset = run_openspec_capture_in_env_root(
        &["config", "list", "--json"],
        &path,
        env_root.path(),
        &[],
    );
    assert!(
        after_reset.success,
        "config list after reset failed: {:?}",
        after_reset
    );
    let reset_json: Value = serde_json::from_str(&after_reset.stdout).unwrap();
    assert_eq!(reset_json["profile"], "core");
    assert!(reset_json["featureFlags"].as_object().unwrap().is_empty());
}

#[test]
fn test_feedback_falls_back_to_manual_url_without_gh() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    // Force the gh-not-found fallback by pointing PATH at an empty dir, so the test never
    // files a real GitHub issue. Feedback files an issue (not telemetry), so it is not gated
    // by OPENSPEC_TELEMETRY.
    let empty_bin = temp_dir.path().join("empty-bin");
    std::fs::create_dir_all(&empty_bin).unwrap();

    let result = run_openspec_capture_with_clean_env(
        &["feedback", "compatibility looks good", "--body", "details"],
        &path,
        &[("PATH", empty_bin.to_str().unwrap())],
    );
    assert!(result.success, "feedback fallback failed: {:?}", result);
    assert!(result.stdout.contains("Manual submission required"));
    assert!(result
        .stdout
        .contains("https://github.com/oonid/OpenSpec-rs/issues/new?"));
    assert!(result.stdout.contains("Feedback%3A%20compatibility"));
}

#[test]
fn test_version_flag() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    let result = run_openspec(&["--version"], &path);
    assert!(result.is_ok(), "version flag failed: {:?}", result);
    let output = result.unwrap();
    assert!(output.contains("openspec"), "version output missing name");
    assert!(
        output.contains(env!("CARGO_PKG_VERSION")),
        "version output missing current package version"
    );
}

#[test]
fn test_no_interactive_flag_is_accepted_by_representative_commands() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    let init_result = run_openspec(&["--no-interactive", "init", "--tools", "none", "."], &path);
    assert!(
        init_result.is_ok(),
        "init with --no-interactive failed: {:?}",
        init_result
    );

    let new_change_result = run_openspec(
        &[
            "--no-interactive",
            "new",
            "change",
            "test-feature",
            "--description",
            "Test",
        ],
        &path,
    );
    assert!(
        new_change_result.is_ok(),
        "new change with --no-interactive failed: {:?}",
        new_change_result
    );

    let validate_result = run_openspec(
        &[
            "--no-interactive",
            "validate",
            "--changes",
            "--concurrency",
            "1",
        ],
        &path,
    );
    assert!(
        validate_result.is_ok(),
        "validate with --no-interactive failed: {:?}",
        validate_result
    );
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
fn test_show_spec_filter_flags_apply_to_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();
    let spec_dir = path.join("openspec/specs/test-spec");

    run_openspec(&["init", "--tools", "none", "."], &path).unwrap();
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(
        spec_dir.join("spec.md"),
        "# Test Spec

## Purpose
Spec overview.

## Requirements

### Requirement: First
The system SHALL do the first thing.

#### Scenario: First
- **WHEN** the first case happens
- **THEN** the first result is produced

### Requirement: Second
The system SHALL do the second thing.

#### Scenario: Second
- **WHEN** the second case happens
- **THEN** the second result is produced
",
    )
    .unwrap();

    let requirements_only = run_openspec(
        &[
            "show",
            "test-spec",
            "--type",
            "spec",
            "--json",
            "--requirements",
        ],
        &path,
    )
    .unwrap();
    let requirements_only_json: Value = serde_json::from_str(&requirements_only).unwrap();
    assert_eq!(requirements_only_json["requirementCount"], 2);
    assert_eq!(
        requirements_only_json["requirements"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(requirements_only_json["requirements"]
        .as_array()
        .unwrap()
        .iter()
        .all(|requirement| requirement["scenarios"].as_array().unwrap().is_empty()));

    let single_requirement = run_openspec(
        &[
            "show",
            "test-spec",
            "--type",
            "spec",
            "--json",
            "--requirement",
            "2",
        ],
        &path,
    )
    .unwrap();
    let single_requirement_json: Value = serde_json::from_str(&single_requirement).unwrap();
    assert_eq!(single_requirement_json["requirementCount"], 1);
    assert_eq!(
        single_requirement_json["requirements"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        single_requirement_json["requirements"][0]["text"],
        "The system SHALL do the second thing."
    );
    assert_eq!(
        single_requirement_json["requirements"][0]["scenarios"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let no_scenarios = run_openspec(
        &[
            "show",
            "test-spec",
            "--type",
            "spec",
            "--json",
            "--no-scenarios",
        ],
        &path,
    )
    .unwrap();
    let no_scenarios_json: Value = serde_json::from_str(&no_scenarios).unwrap();
    assert_eq!(no_scenarios_json["requirementCount"], 2);
    assert!(no_scenarios_json["requirements"]
        .as_array()
        .unwrap()
        .iter()
        .all(|requirement| requirement["scenarios"].as_array().unwrap().is_empty()));

    let plain_text = run_openspec_capture(
        &["show", "test-spec", "--type", "spec", "--requirements"],
        &path,
    );
    assert!(!plain_text.success, "show without --json should fail");
    assert!(plain_text.stderr.contains("require --json"));
}

#[test]
fn test_show_change_warns_when_spec_filter_flags_are_used() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();

    run_openspec(&["init", "--tools", "none", "."], &path).unwrap();
    run_openspec(
        &["new", "change", "test-feature", "--description", "Test"],
        &path,
    )
    .unwrap();

    let result = run_openspec_capture(&["show", "test-feature", "--json", "--requirements"], &path);
    assert!(
        result.success,
        "show change should still succeed: {:?}",
        result
    );
    assert!(result
        .stderr
        .contains("Ignoring spec-only flags for change 'test-feature'"));
    assert!(result.stdout.contains("\"id\": \"test-feature\""));
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
fn test_embedded_schema_commands_work_from_repo_root() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert_embedded_schema_commands_work(&path);
}

#[test]
fn test_embedded_schema_commands_work_from_isolated_dir() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_path_buf();
    assert_embedded_schema_commands_work(&path);
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
