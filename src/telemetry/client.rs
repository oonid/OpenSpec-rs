use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use uuid::Uuid;

const POSTHOG_API_KEY: &str = "phc_Hthu8YvaIJ9QaFKyTG4TbVwkbd5ktcAFzVTKeMmoW2g";
const POSTHOG_HOST: &str = "https://edge.openspec.dev";

static ANONYMOUS_ID: OnceLock<String> = OnceLock::new();
static PENDING_EVENTS: OnceLock<Mutex<Vec<PostHogEvent>>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TelemetryConfig {
    #[serde(rename = "anonymousId", skip_serializing_if = "Option::is_none")]
    pub anonymous_id: Option<String>,
    #[serde(rename = "noticeSeen", skip_serializing_if = "Option::is_none")]
    pub notice_seen: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<TelemetryConfig>,
}

pub fn is_telemetry_enabled() -> bool {
    if std::env::var("OPENSPEC_TELEMETRY").ok().as_deref() == Some("0") {
        return false;
    }

    if std::env::var("DO_NOT_TRACK").ok().as_deref() == Some("1") {
        return false;
    }

    if std::env::var("CI").ok().as_deref() == Some("true") {
        return false;
    }

    true
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openspec")
        .join("config.json")
}

fn read_config() -> GlobalConfig {
    let config_path = get_config_path();
    match std::fs::read_to_string(&config_path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => GlobalConfig::default(),
    }
}

fn write_config(config: &GlobalConfig) {
    let config_path = get_config_path();
    if let Some(parent) = config_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(
        &config_path,
        serde_json::to_string_pretty(config).unwrap_or_default(),
    );
}

pub fn get_telemetry_config() -> TelemetryConfig {
    read_config().telemetry.unwrap_or_default()
}

pub fn update_telemetry_config(updates: TelemetryConfig) {
    let mut config = read_config();
    let existing = config.telemetry.unwrap_or_default();
    config.telemetry = Some(TelemetryConfig {
        anonymous_id: updates.anonymous_id.or(existing.anonymous_id),
        notice_seen: updates.notice_seen.or(existing.notice_seen),
    });
    write_config(&config);
}

pub fn get_or_create_anonymous_id() -> String {
    if let Some(id) = ANONYMOUS_ID.get() {
        return id.clone();
    }

    let config = get_telemetry_config();
    if let Some(id) = config.anonymous_id {
        let _ = ANONYMOUS_ID.set(id.clone());
        return id;
    }

    let id = Uuid::new_v4().to_string();
    update_telemetry_config(TelemetryConfig {
        anonymous_id: Some(id.clone()),
        ..Default::default()
    });
    let _ = ANONYMOUS_ID.set(id.clone());
    id
}

#[derive(Serialize)]
struct PostHogEvent {
    api_key: String,
    event: String,
    distinct_id: String,
    properties: PostHogProperties,
}

#[derive(Serialize)]
struct PostHogProperties {
    command: String,
    version: String,
    surface: String,
    #[serde(rename = "$ip")]
    ip: Option<String>,
}

pub fn track_command(command_name: &str, version: &str) {
    if !is_telemetry_enabled() {
        return;
    }

    let user_id = get_or_create_anonymous_id();

    let event = PostHogEvent {
        api_key: POSTHOG_API_KEY.to_string(),
        event: "command_executed".to_string(),
        distinct_id: user_id,
        properties: PostHogProperties {
            command: command_name.to_string(),
            version: version.to_string(),
            surface: "cli".to_string(),
            ip: None,
        },
    };

    if let Some(pending) = PENDING_EVENTS.get() {
        if let Ok(mut events) = pending.lock() {
            events.push(event);
            return;
        }
    }

    let _ = ureq::post(&format!("{}/capture/", POSTHOG_HOST))
        .set("Content-Type", "application/json")
        .send_json(&event);
}

pub fn init_pending_events() {
    let _ = PENDING_EVENTS.get_or_init(|| Mutex::new(Vec::new()));
}

pub fn flush_and_shutdown() {
    if !is_telemetry_enabled() {
        return;
    }

    if let Some(pending) = PENDING_EVENTS.get() {
        if let Ok(mut events) = pending.lock() {
            for event in events.drain(..) {
                let _ = ureq::post(&format!("{}/capture/", POSTHOG_HOST))
                    .set("Content-Type", "application/json")
                    .send_json(&event);
            }
        }
    }
}

pub fn maybe_show_telemetry_notice() {
    if !is_telemetry_enabled() {
        return;
    }

    let config = get_telemetry_config();
    if config.notice_seen.unwrap_or(false) {
        return;
    }

    eprintln!("Note: OpenSpec collects anonymous usage stats. Opt out: OPENSPEC_TELEMETRY=0");

    update_telemetry_config(TelemetryConfig {
        notice_seen: Some(true),
        ..Default::default()
    });
}
