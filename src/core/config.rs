use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::error::{OpenSpecError, Result};

pub const OPENSPEC_DIR_NAME: &str = "openspec";
pub const CONFIG_FILE_NAME: &str = "config.yaml";
pub const GLOBAL_CONFIG_FILE_NAME: &str = "config.json";

pub fn xdg_config_dir() -> PathBuf {
    if let Some(xdg_config) = std::env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(xdg_config).join("openspec")
    } else if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("openspec")
    } else {
        PathBuf::from(".config").join("openspec")
    }
}

pub fn xdg_data_dir() -> PathBuf {
    if let Some(xdg_data) = std::env::var_os("XDG_DATA_HOME") {
        PathBuf::from(xdg_data).join("openspec")
    } else if let Some(data_dir) = dirs::data_local_dir() {
        data_dir.join("openspec")
    } else {
        PathBuf::from(".local").join("share").join("openspec")
    }
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
pub struct GlobalConfig {
    #[serde(default = "default_profile")]
    pub profile: String,
    #[serde(default = "default_delivery")]
    pub delivery: String,
    #[serde(default = "default_workflows")]
    pub workflows: Vec<String>,
    #[serde(default)]
    pub feature_flags: HashMap<String, bool>,
    #[serde(default)]
    pub telemetry: Option<TelemetryConfig>,
}

fn default_profile() -> String {
    "core".to_string()
}

fn default_delivery() -> String {
    "both".to_string()
}

fn default_workflows() -> Vec<String> {
    vec![
        "propose".to_string(),
        "explore".to_string(),
        "apply".to_string(),
        "archive".to_string(),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    #[serde(default)]
    pub anonymous_id: Option<String>,
    #[serde(default)]
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

        let content = serde_json::to_string_pretty(config)
            .map_err(|e| OpenSpecError::Custom(format!("JSON serialization error: {}", e)))?;

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
