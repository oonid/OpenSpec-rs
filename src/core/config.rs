use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::error::{OpenSpecError, Result};

pub const OPENSPEC_DIR_NAME: &str = "openspec";
pub const CONFIG_FILE_NAME: &str = "config.yaml";
pub const GLOBAL_CONFIG_FILE_NAME: &str = "config.json";

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// Resolve a global config/data directory, mirroring upstream `getGlobalConfigDir`/
/// `getGlobalDataDir` exactly so the Rust binary and the npm CLI share locations:
/// the `*_HOME` env var wins on all platforms; otherwise Windows uses its env dir
/// (with an `AppData/...` fallback) and macOS/Linux both use a dotfile path under `$HOME`.
///
/// NOTE: unlike the `dirs` crate, macOS uses `~/.config` and `~/.local/share` here (not
/// `~/Library/Application Support`) — that is intentional, to match upstream.
fn resolve_global_dir(
    xdg_override: Option<&std::ffi::OsStr>,
    win_override: Option<&std::ffi::OsStr>,
    home: &Path,
    is_windows: bool,
    win_fallback: &[&str],
    unix_fallback: &[&str],
) -> PathBuf {
    if let Some(x) = xdg_override {
        return Path::new(x).join(OPENSPEC_DIR_NAME);
    }
    if is_windows {
        if let Some(w) = win_override {
            return Path::new(w).join(OPENSPEC_DIR_NAME);
        }
        let mut p = home.to_path_buf();
        p.extend(win_fallback);
        return p.join(OPENSPEC_DIR_NAME);
    }
    let mut p = home.to_path_buf();
    p.extend(unix_fallback);
    p.join(OPENSPEC_DIR_NAME)
}

pub fn xdg_config_dir() -> PathBuf {
    resolve_global_dir(
        std::env::var_os("XDG_CONFIG_HOME").as_deref(),
        std::env::var_os("APPDATA").as_deref(),
        &home_dir(),
        cfg!(target_os = "windows"),
        &["AppData", "Roaming"],
        &[".config"],
    )
}

pub fn xdg_data_dir() -> PathBuf {
    resolve_global_dir(
        std::env::var_os("XDG_DATA_HOME").as_deref(),
        std::env::var_os("LOCALAPPDATA").as_deref(),
        &home_dir(),
        cfg!(target_os = "windows"),
        &["AppData", "Local"],
        &[".local", "share"],
    )
}

pub fn xdg_config_path() -> PathBuf {
    xdg_config_dir().join(GLOBAL_CONFIG_FILE_NAME)
}

pub fn xdg_data_path(relative: &str) -> PathBuf {
    xdg_data_dir().join(relative)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default)]
    pub schema: String,
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub rules: HashMap<String, Vec<String>>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            schema: "spec-driven".to_string(),
            context: None,
            rules: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalConfig {
    #[serde(default = "default_profile")]
    pub profile: String,
    #[serde(default = "default_delivery")]
    pub delivery: String,
    #[serde(default = "default_workflows")]
    pub workflows: Vec<String>,
    // Serializes as `featureFlags` to match upstream config.json; `feature_flags` alias
    // keeps reading configs written by earlier Rust versions.
    #[serde(default, alias = "feature_flags")]
    pub feature_flags: HashMap<String, bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<TelemetryConfig>,
}

fn default_profile() -> String {
    "core".to_string()
}

fn default_delivery() -> String {
    "both".to_string()
}

fn default_workflows() -> Vec<String> {
    // Matches upstream CORE_WORKFLOWS (profiles.ts): the 'core' profile installs exactly
    // these five workflows. `verify` and others are not part of core.
    vec![
        "propose".to_string(),
        "explore".to_string(),
        "apply".to_string(),
        "sync".to_string(),
        "archive".to_string(),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryConfig {
    // camelCase on disk (`anonymousId`, `noticeSeen`) to match upstream; snake_case aliases
    // keep reading configs written by earlier Rust versions.
    #[serde(
        default,
        alias = "anonymous_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub anonymous_id: Option<String>,
    #[serde(default, alias = "notice_seen")]
    pub notice_seen: bool,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            profile: default_profile(),
            delivery: default_delivery(),
            workflows: default_workflows(),
            feature_flags: HashMap::new(),
            telemetry: None,
        }
    }
}

pub struct ConfigManager {
    project_root: Option<PathBuf>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            project_root: find_project_root(),
        }
    }

    pub fn with_root(project_root: PathBuf) -> Self {
        Self {
            project_root: Some(project_root),
        }
    }

    pub fn project_root(&self) -> Option<&Path> {
        self.project_root.as_deref()
    }

    pub fn openspec_dir(&self) -> Option<PathBuf> {
        self.project_root
            .as_ref()
            .map(|p| p.join(OPENSPEC_DIR_NAME))
    }

    pub fn load_project_config(&self) -> Result<ProjectConfig> {
        let openspec_dir = self.openspec_dir().ok_or(OpenSpecError::NotInitialized)?;
        let config_path = openspec_dir.join(CONFIG_FILE_NAME);

        if !config_path.exists() {
            return Ok(ProjectConfig::default());
        }

        let content = std::fs::read_to_string(&config_path).map_err(|e| OpenSpecError::IoRead {
            path: config_path.clone(),
            source: e,
        })?;

        let config: ProjectConfig = serde_yaml::from_str(&content).map_err(|e| {
            OpenSpecError::yaml_parse(config_path.display().to_string(), e.to_string())
        })?;

        Ok(config)
    }

    pub fn save_project_config(&self, config: &ProjectConfig) -> Result<()> {
        let openspec_dir = self.openspec_dir().ok_or(OpenSpecError::NotInitialized)?;
        let config_path = openspec_dir.join(CONFIG_FILE_NAME);

        let content = serde_yaml::to_string(config)
            .map_err(|e| OpenSpecError::yaml_parse("config", e.to_string()))?;

        std::fs::write(&config_path, content).map_err(|e| OpenSpecError::IoWrite {
            path: config_path,
            source: e,
        })?;

        Ok(())
    }

    pub fn global_config_path() -> PathBuf {
        xdg_config_path()
    }

    pub fn load_global_config() -> GlobalConfig {
        let config_path = Self::global_config_path();

        if !config_path.exists() {
            return GlobalConfig::default();
        }

        let content = match std::fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(_) => return GlobalConfig::default(),
        };

        let parsed: serde_json::Result<GlobalConfig> = serde_json::from_str(&content);

        match parsed {
            Ok(loaded) => GlobalConfig {
                profile: loaded.profile,
                delivery: loaded.delivery,
                workflows: loaded.workflows,
                feature_flags: loaded.feature_flags,
                telemetry: loaded.telemetry,
            },
            Err(_) => GlobalConfig::default(),
        }
    }

    pub fn save_global_config(config: &GlobalConfig) -> Result<()> {
        let config_path = Self::global_config_path();

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| OpenSpecError::IoWrite {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        // Trailing newline to match upstream (`JSON.stringify(...) + '\n'`).
        let content = format!(
            "{}\n",
            serde_json::to_string_pretty(config)
                .map_err(|e| OpenSpecError::Custom(format!("JSON serialization error: {}", e)))?
        );

        std::fs::write(&config_path, content).map_err(|e| OpenSpecError::IoWrite {
            path: config_path.clone(),
            source: e,
        })?;

        Ok(())
    }
}

fn find_project_root() -> Option<PathBuf> {
    let current = std::env::current_dir().ok()?;
    let mut path = current.as_path();

    loop {
        let openspec_dir = path.join(OPENSPEC_DIR_NAME);
        if openspec_dir.is_dir() {
            return Some(path.to_path_buf());
        }

        path = path.parent()?;
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Best-effort, one-time migration for macOS users whose global config lived under the old
/// `~/Library/Application Support/openspec` location (the `dirs` crate default) before
/// `xdg_config_dir`/`xdg_data_dir` were aligned with upstream's `~/.config`/`~/.local/share`.
///
/// Copies the legacy global config file to its new home if the new one does not exist yet, so
/// macOS users keep their telemetry preference / anonymous id. No-op on other platforms and
/// when `XDG_CONFIG_HOME` is set. Non-fatal: any error is ignored.
pub fn migrate_legacy_macos_global_config() {
    #[cfg(target_os = "macos")]
    {
        if std::env::var_os("XDG_CONFIG_HOME").is_some() {
            return;
        }
        if let Some(legacy_base) = dirs::config_dir() {
            let legacy_config = legacy_base
                .join(OPENSPEC_DIR_NAME)
                .join(GLOBAL_CONFIG_FILE_NAME);
            let new_config = xdg_config_path();
            if legacy_config.exists() && !new_config.exists() {
                if let Some(parent) = new_config.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::copy(&legacy_config, &new_config);
            }
        }
    }
}

#[cfg(test)]
mod dir_resolution_tests {
    use super::*;
    use std::ffi::OsStr;
    use std::path::Path;

    #[test]
    fn xdg_override_wins_on_all_platforms() {
        let home = Path::new("/home/u");
        for is_windows in [true, false] {
            let dir = resolve_global_dir(
                Some(OsStr::new("/custom/xdg")),
                Some(OsStr::new("/win/appdata")),
                home,
                is_windows,
                &["AppData", "Roaming"],
                &[".config"],
            );
            assert_eq!(dir, Path::new("/custom/xdg/openspec"));
        }
    }

    #[test]
    fn unix_and_macos_use_dotfile_path() {
        // is_windows = false covers both Linux and macOS — matches upstream (no Library/... path).
        let dir = resolve_global_dir(
            None,
            None,
            Path::new("/Users/u"),
            false,
            &["AppData", "Local"],
            &[".local", "share"],
        );
        assert_eq!(dir, Path::new("/Users/u/.local/share/openspec"));
    }

    #[test]
    fn windows_uses_env_then_fallback() {
        let home = Path::new("C:\\Users\\u");
        let with_env = resolve_global_dir(
            None,
            Some(OsStr::new("D:\\AppData\\Local")),
            home,
            true,
            &["AppData", "Local"],
            &[".local", "share"],
        );
        assert_eq!(with_env, Path::new("D:\\AppData\\Local").join("openspec"));

        let fallback = resolve_global_dir(
            None,
            None,
            home,
            true,
            &["AppData", "Local"],
            &[".local", "share"],
        );
        assert_eq!(
            fallback,
            home.join("AppData").join("Local").join("openspec")
        );
    }
}

#[cfg(test)]
mod global_config_serde_tests {
    use super::*;

    #[test]
    fn global_config_serializes_camelcase_to_match_upstream() {
        let mut flags = HashMap::new();
        flags.insert("beta".to_string(), true);
        let cfg = GlobalConfig {
            profile: "core".to_string(),
            delivery: "both".to_string(),
            workflows: vec!["propose".to_string()],
            feature_flags: flags,
            telemetry: Some(TelemetryConfig {
                anonymous_id: Some("abc".to_string()),
                notice_seen: true,
            }),
        };
        let json = serde_json::to_string(&cfg).unwrap();
        assert!(json.contains("\"featureFlags\""), "{json}");
        assert!(json.contains("\"anonymousId\""), "{json}");
        assert!(json.contains("\"noticeSeen\""), "{json}");
        assert!(!json.contains("feature_flags"));
        assert!(!json.contains("anonymous_id"));
    }

    #[test]
    fn telemetry_omitted_when_none() {
        let cfg = GlobalConfig {
            profile: "core".to_string(),
            delivery: "both".to_string(),
            workflows: vec![],
            feature_flags: HashMap::new(),
            telemetry: None,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        assert!(!json.contains("telemetry"), "{json}");
    }

    #[test]
    fn reads_camelcase_and_legacy_snake_case() {
        // Upstream / new Rust camelCase.
        let camel: GlobalConfig = serde_json::from_str(
            r#"{"featureFlags":{"beta":true},"telemetry":{"anonymousId":"x","noticeSeen":true}}"#,
        )
        .unwrap();
        assert_eq!(camel.feature_flags.get("beta"), Some(&true));
        assert_eq!(
            camel.telemetry.as_ref().unwrap().anonymous_id.as_deref(),
            Some("x")
        );

        // Legacy snake_case written by earlier Rust versions still reads.
        let snake: GlobalConfig = serde_json::from_str(
            r#"{"feature_flags":{"beta":true},"telemetry":{"anonymous_id":"x","notice_seen":true}}"#,
        )
        .unwrap();
        assert_eq!(snake.feature_flags.get("beta"), Some(&true));
        assert!(snake.telemetry.as_ref().unwrap().notice_seen);
    }
}
