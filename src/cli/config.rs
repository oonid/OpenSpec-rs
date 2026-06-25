use crate::core::config::{ConfigManager, GlobalConfig};
use crate::core::error::{OpenSpecError, Result};
use crate::templates::skills::CORE_WORKFLOWS;
use serde_json::{Map, Value};
use std::ffi::OsString;
use std::process::Command;

pub fn run_config_path() -> Result<()> {
    println!("{}", ConfigManager::global_config_path().display());
    Ok(())
}

pub fn run_config_list(json: bool) -> Result<()> {
    let config = ConfigManager::load_global_config();

    if json {
        println!("{}", serde_json::to_string_pretty(&config)?);
        return Ok(());
    }

    let yaml = serde_yaml::to_string(&config)?;
    print_yaml_without_document_marker(&yaml);
    Ok(())
}

pub fn run_config_get(key: &str) -> Result<()> {
    let config = ConfigManager::load_global_config();
    let value = get_nested_value(&config, key);

    match value {
        Some(v) => {
            if let Some(s) = v.as_str() {
                println!("{}", s);
            } else if v.is_object() || v.is_array() {
                println!("{}", serde_json::to_string_pretty(&v)?);
            } else {
                println!("{}", v);
            }
            Ok(())
        }
        None => {
            eprintln!("Key '{}' not found", key);
            std::process::exit(1);
        }
    }
}

pub fn run_config_set(key: &str, value: &str) -> Result<()> {
    let mut config = ConfigManager::load_global_config();
    set_nested_value(&mut config, key, value)?;
    ConfigManager::save_global_config(&config)?;
    println!("Set {} = {}", key, value);
    Ok(())
}

pub fn run_config_unset(key: &str) -> Result<()> {
    let mut config = ConfigManager::load_global_config();
    let existed = unset_nested_value(&mut config, key)?;

    if existed {
        ConfigManager::save_global_config(&config)?;
        println!("Unset {} (reverted to default)", key);
    } else {
        println!("Key \"{}\" was not set", key);
    }

    Ok(())
}

pub fn run_config_reset(all: bool) -> Result<()> {
    // Mirror upstream: refuse to wipe the config unless --all is explicitly passed.
    if !all {
        return Err(OpenSpecError::Custom(
            "--all flag is required for reset. Usage: openspec config reset --all".to_string(),
        ));
    }
    ConfigManager::save_global_config(&GlobalConfig::default())?;
    println!("Configuration reset to defaults");
    Ok(())
}

pub fn run_config_profile(preset: Option<&str>) -> Result<()> {
    match preset {
        None => {
            let config = ConfigManager::load_global_config();
            println!("{}", config.profile);
            Ok(())
        }
        Some("core") => {
            let mut config = ConfigManager::load_global_config();
            apply_profile_preset(&mut config, "core");
            ConfigManager::save_global_config(&config)?;
            println!("core");
            Ok(())
        }
        Some(preset) => Err(OpenSpecError::Custom(format!(
            "Unknown profile preset \"{}\". Available presets: core",
            preset
        ))),
    }
}

pub fn run_config_edit() -> Result<()> {
    let config_path = ConfigManager::global_config_path();
    ensure_global_config_exists(&config_path)?;

    let editor = resolve_config_editor(
        std::env::var_os("VISUAL")
            .as_deref()
            .and_then(|s| s.to_str()),
        std::env::var_os("EDITOR")
            .as_deref()
            .and_then(|s| s.to_str()),
        cfg!(windows),
    );

    let status = Command::new(&editor.program)
        .args(&editor.args)
        .arg(&config_path)
        .status()
        .map_err(|err| {
            OpenSpecError::Custom(format!(
                "Failed to launch editor '{}': {}",
                editor.program, err
            ))
        })?;

    if !status.success() {
        return Err(OpenSpecError::Custom(format!(
            "Editor '{}' exited with status {}",
            editor.program, status
        )));
    }

    Ok(())
}

fn apply_profile_preset(config: &mut GlobalConfig, preset: &str) {
    if preset == "core" {
        config.profile = "core".to_string();
        config.workflows = CORE_WORKFLOWS
            .iter()
            .map(|workflow| workflow.to_string())
            .collect();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedEditor {
    pub program: String,
    pub args: Vec<OsString>,
}

pub fn resolve_config_editor(
    visual: Option<&str>,
    editor: Option<&str>,
    is_windows: bool,
) -> ResolvedEditor {
    let program = visual
        .filter(|value| !value.trim().is_empty())
        .or_else(|| editor.filter(|value| !value.trim().is_empty()))
        .map(|value| value.to_string())
        .unwrap_or_else(|| {
            if is_windows {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });

    ResolvedEditor {
        program,
        args: Vec::new(),
    }
}

fn ensure_global_config_exists(config_path: &std::path::Path) -> Result<()> {
    if config_path.exists() {
        return Ok(());
    }

    ConfigManager::save_global_config(&GlobalConfig::default())
}

fn print_yaml_without_document_marker(yaml: &str) {
    if let Some(stripped) = yaml.strip_prefix("---\n") {
        print!("{}", stripped);
    } else {
        print!("{}", yaml);
    }
}

fn get_nested_value(config: &GlobalConfig, key: &str) -> Option<Value> {
    let json = serde_json::to_value(config).ok()?;
    let normalized = normalize_key_path(key);
    let parts: Vec<&str> = normalized.split('.').collect();
    let mut current = &json;

    for part in &parts {
        if let Some(obj) = current.as_object() {
            current = obj.get(*part)?;
        } else {
            return None;
        }
    }

    Some(current.clone())
}

fn set_nested_value(config: &mut GlobalConfig, key: &str, value: &str) -> Result<()> {
    let normalized = normalize_key_path(key);
    // Validate enum-constrained values, mirroring upstream config-schema (profile/delivery).
    match normalized.as_str() {
        "profile" if !["core", "custom"].contains(&value) => {
            return Err(OpenSpecError::Custom(format!(
                "Invalid value for 'profile': '{}'. Allowed: core, custom",
                value
            )));
        }
        "delivery" if !["both", "skills", "commands"].contains(&value) => {
            return Err(OpenSpecError::Custom(format!(
                "Invalid value for 'delivery': '{}'. Allowed: both, skills, commands",
                value
            )));
        }
        _ => {}
    }
    let coerced = if normalized == "workflows" {
        Value::Array(
            value
                .split(',')
                .map(|item| Value::String(item.trim().to_string()))
                .collect(),
        )
    } else {
        coerce_value(value)
    };
    let mut json = serde_json::to_value(&*config)?;
    set_nested_json_value(&mut json, &normalized, coerced)?;
    *config = serde_json::from_value(json).map_err(|err| {
        OpenSpecError::Custom(format!(
            "Invalid configuration value for '{}': {}",
            key, err
        ))
    })?;
    Ok(())
}

fn unset_nested_value(config: &mut GlobalConfig, key: &str) -> Result<bool> {
    let mut json = serde_json::to_value(&*config)?;
    let existed = delete_nested_json_value(&mut json, &normalize_key_path(key));
    *config = serde_json::from_value(json).map_err(|err| {
        OpenSpecError::Custom(format!(
            "Invalid configuration after unsetting '{}': {}",
            key, err
        ))
    })?;
    Ok(existed)
}

fn normalize_key_path(key: &str) -> String {
    key.split('.')
        .map(|part| match part {
            "feature_flags" => "featureFlags",
            "anonymous_id" => "anonymousId",
            "notice_seen" => "noticeSeen",
            other => other,
        })
        .collect::<Vec<_>>()
        .join(".")
}

fn coerce_value(value: &str) -> Value {
    if value == "true" {
        Value::Bool(true)
    } else if value == "false" {
        Value::Bool(false)
    } else if let Ok(n) = value.parse::<i64>() {
        Value::Number(n.into())
    } else {
        Value::String(value.to_string())
    }
}

fn set_nested_json_value(root: &mut Value, key: &str, value: Value) -> Result<()> {
    let parts: Vec<&str> = key.split('.').filter(|part| !part.is_empty()).collect();
    if parts.is_empty() {
        return Err(OpenSpecError::Custom(
            "Configuration key cannot be empty".to_string(),
        ));
    }

    let mut current = root
        .as_object_mut()
        .ok_or_else(|| OpenSpecError::Custom("Configuration root must be an object".to_string()))?;

    for part in &parts[..parts.len() - 1] {
        let entry = current
            .entry((*part).to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if !entry.is_object() {
            *entry = Value::Object(Map::new());
        }
        current = entry.as_object_mut().expect("entry forced to object");
    }

    current.insert(parts[parts.len() - 1].to_string(), value);
    Ok(())
}

fn delete_nested_json_value(root: &mut Value, key: &str) -> bool {
    let parts: Vec<&str> = key.split('.').filter(|part| !part.is_empty()).collect();
    if parts.is_empty() {
        return false;
    }

    let Some(object) = root.as_object_mut() else {
        return false;
    };

    delete_nested_from_map(object, &parts)
}

fn delete_nested_from_map(map: &mut Map<String, Value>, parts: &[&str]) -> bool {
    if parts.len() == 1 {
        return map.remove(parts[0]).is_some();
    }

    let Some(child) = map.get_mut(parts[0]) else {
        return false;
    };
    let Some(child_map) = child.as_object_mut() else {
        return false;
    };

    let existed = delete_nested_from_map(child_map, &parts[1..]);
    if existed && child_map.is_empty() {
        map.remove(parts[0]);
    }
    existed
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    struct EnvGuard {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &std::path::Path) -> Self {
            let previous = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    #[test]
    fn config_list_yaml_uses_camelcase_keys() {
        let yaml = serde_yaml::to_string(&GlobalConfig::default()).unwrap();
        assert!(yaml.contains("featureFlags"));
        assert!(!yaml.contains("feature_flags"));
    }

    #[test]
    fn set_and_get_nested_camelcase_and_legacy_aliases() {
        let mut config = GlobalConfig::default();
        set_nested_value(&mut config, "feature_flags.beta", "true").unwrap();
        set_nested_value(&mut config, "telemetry.anonymous_id", "abc").unwrap();

        assert_eq!(
            get_nested_value(&config, "featureFlags.beta"),
            Some(Value::Bool(true))
        );
        assert_eq!(
            get_nested_value(&config, "telemetry.anonymousId"),
            Some(Value::String("abc".to_string()))
        );
    }

    #[test]
    fn unset_nested_value_removes_empty_parent_objects() {
        let mut config = GlobalConfig::default();
        set_nested_value(&mut config, "telemetry.anonymousId", "abc").unwrap();

        let existed = unset_nested_value(&mut config, "telemetry.anonymousId").unwrap();

        assert!(existed);
        assert_eq!(get_nested_value(&config, "telemetry.anonymousId"), None);
    }

    #[test]
    fn unset_missing_key_reports_false() {
        let mut config = GlobalConfig::default();
        let existed = unset_nested_value(&mut config, "featureFlags.beta").unwrap();
        assert!(!existed);
    }

    #[test]
    fn profile_preset_sets_core_and_workflows() {
        let mut config = GlobalConfig {
            profile: "custom".to_string(),
            workflows: vec!["custom-one".to_string()],
            ..GlobalConfig::default()
        };

        apply_profile_preset(&mut config, "core");

        assert_eq!(config.profile, "core");
        assert_eq!(
            config.workflows,
            CORE_WORKFLOWS
                .iter()
                .map(|workflow| workflow.to_string())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn resolve_config_editor_prefers_visual_then_editor_then_platform_default() {
        let visual = resolve_config_editor(Some("nano"), Some("vim"), false);
        assert_eq!(visual.program, "nano");

        let editor = resolve_config_editor(None, Some("vim"), false);
        assert_eq!(editor.program, "vim");

        let unix_default = resolve_config_editor(None, None, false);
        assert_eq!(unix_default.program, "vi");

        let windows_default = resolve_config_editor(None, None, true);
        assert_eq!(windows_default.program, "notepad");
    }

    #[test]
    fn config_round_trip_set_get_unset_reset() {
        let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let _env_guard = EnvGuard::set("XDG_CONFIG_HOME", temp_dir.path());

        run_config_set("profile", "custom").unwrap();
        run_config_set("delivery", "skills").unwrap();
        run_config_set("workflows", "alpha,beta").unwrap();
        run_config_set("featureFlags.beta", "true").unwrap();
        run_config_set("telemetry.anonymousId", "anon-1").unwrap();

        let config = ConfigManager::load_global_config();
        assert_eq!(
            get_nested_value(&config, "profile"),
            Some(Value::String("custom".to_string()))
        );
        assert_eq!(
            get_nested_value(&config, "delivery"),
            Some(Value::String("skills".to_string()))
        );
        assert_eq!(
            get_nested_value(&config, "workflows"),
            Some(Value::Array(vec![
                Value::String("alpha".to_string()),
                Value::String("beta".to_string()),
            ]))
        );
        assert_eq!(
            get_nested_value(&config, "featureFlags.beta"),
            Some(Value::Bool(true))
        );
        assert_eq!(
            get_nested_value(&config, "telemetry.anonymousId"),
            Some(Value::String("anon-1".to_string()))
        );

        run_config_unset("telemetry.anonymousId").unwrap();
        let config = ConfigManager::load_global_config();
        assert_eq!(get_nested_value(&config, "telemetry.anonymousId"), None);

        run_config_reset(true).unwrap();
        let config = ConfigManager::load_global_config();
        let default = GlobalConfig::default();
        assert_eq!(config.profile, default.profile);
        assert_eq!(config.delivery, default.delivery);
        assert_eq!(config.workflows, default.workflows);
        assert_eq!(config.feature_flags, default.feature_flags);
        assert!(config.telemetry.is_none());
    }

    #[test]
    fn set_rejects_invalid_enum_values() {
        let mut config = GlobalConfig::default();
        assert!(set_nested_value(&mut config, "delivery", "bogus").is_err());
        assert!(set_nested_value(&mut config, "profile", "nope").is_err());
        // valid values still accepted
        assert!(set_nested_value(&mut config, "delivery", "skills").is_ok());
        assert_eq!(config.delivery, "skills");
        assert!(set_nested_value(&mut config, "profile", "custom").is_ok());
        assert_eq!(config.profile, "custom");
    }

    #[test]
    fn reset_requires_all_flag() {
        // Without --all, reset must error before touching the config (mirrors upstream).
        let err = run_config_reset(false).unwrap_err();
        assert!(err.to_string().contains("--all"));
    }
}
