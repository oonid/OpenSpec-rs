use crate::core::config::{ConfigManager, GlobalConfig};
use crate::core::error::{OpenSpecError, Result};

pub fn run_config(set: Option<&str>, get: Option<&str>, list: bool) -> Result<()> {
    if list || (set.is_none() && get.is_none()) {
        run_config_list()
    } else if let Some(key) = get {
        run_config_get(key)
    } else if let Some(kv) = set {
        run_config_set(kv)
    } else {
        run_config_list()
    }
}

fn run_config_list() -> Result<()> {
    let config = ConfigManager::load_global_config();
    let config_path = ConfigManager::global_config_path();

    println!("Config file: {}", config_path.display());
    println!();

    println!("Profile settings:");
    println!("  profile: {}", config.profile);
    println!("  delivery: {}", config.delivery);
    println!("  workflows: {}", config.workflows.join(", "));

    if !config.feature_flags.is_empty() {
        println!();
        println!("Feature flags:");
        for (key, value) in &config.feature_flags {
            println!("  {}: {}", key, value);
        }
    }

    if let Some(telemetry) = &config.telemetry {
        println!();
        println!("Telemetry:");
        if let Some(id) = &telemetry.anonymous_id {
            println!("  anonymous_id: {}", id);
        }
        println!("  notice_seen: {}", telemetry.notice_seen);
    }

    Ok(())
}

fn run_config_get(key: &str) -> Result<()> {
    let config = ConfigManager::load_global_config();
    let value = get_nested_value(&config, key);

    match value {
        Some(v) => {
            if v.is_object() {
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

fn run_config_set(kv: &str) -> Result<()> {
    let parts: Vec<&str> = kv.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(OpenSpecError::Custom(
            "Invalid format. Use: --set key=value".to_string(),
        ));
    }

    let key = parts[0];
    let value = parts[1];

    let mut config = ConfigManager::load_global_config();
    set_nested_value(&mut config, key, value);

    ConfigManager::save_global_config(&config)?;

    println!("Set {} = {}", key, value);

    Ok(())
}

fn get_nested_value(config: &GlobalConfig, key: &str) -> Option<serde_json::Value> {
    let json = serde_json::to_value(config).ok()?;

    let parts: Vec<&str> = key.split('.').collect();
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

fn set_nested_value(config: &mut GlobalConfig, key: &str, value: &str) {
    match key {
        "profile" => {
            config.profile = value.to_string();
        }
        "delivery" => {
            config.delivery = value.to_string();
        }
        "workflows" => {
            config.workflows = value.split(',').map(|s| s.trim().to_string()).collect();
        }
        _ => {
            let coerced: serde_json::Value = if value == "true" {
                serde_json::Value::Bool(true)
            } else if value == "false" {
                serde_json::Value::Bool(false)
            } else if let Ok(n) = value.parse::<i64>() {
                serde_json::Value::Number(n.into())
            } else {
                serde_json::Value::String(value.to_string())
            };

            if key.starts_with("feature_flags.") {
                let flag_name = key.strip_prefix("feature_flags.").unwrap();
                if let Some(b) = coerced.as_bool() {
                    config.feature_flags.insert(flag_name.to_string(), b);
                }
            }
        }
    }
}
