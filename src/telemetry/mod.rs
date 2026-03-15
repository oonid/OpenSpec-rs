pub mod client;

pub use client::{
    flush_and_shutdown, get_or_create_anonymous_id, get_telemetry_config, init_pending_events,
    is_telemetry_enabled, maybe_show_telemetry_notice, track_command, update_telemetry_config,
    GlobalConfig, TelemetryConfig,
};
