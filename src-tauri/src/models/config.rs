use serde::{Deserialize, Serialize};

use super::gateway_config::GatewayConfig;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ClgoConfig {
    pub scheduled_wakeups: Vec<ScheduledWakeup>,
    pub heartbeat: HeartbeatConfig,
    pub advanced: AdvancedSettings,
    pub gateway: GatewayConfig,
    pub mascot: super::mascot::MascotSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AdvancedSettings {
    pub autostart: bool,
    pub start_hidden: bool,
    pub show_tray: bool,
    pub default_model: String,
    pub keep_alive: String,
    pub allowed_paths: Vec<String>,
    pub session_outputs_directory: String,
    pub hardware_accel: String,
    pub multi_model: bool,
    pub show_gpu_status: bool,
    pub compression_enabled: bool,
    pub compression_threshold: u8,
    pub response_language: String,
    pub link_preview_enabled: bool,
    pub ollama_setup_skipped: bool,
    pub onboarding_completed: bool,
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            autostart: false,
            start_hidden: false,
            show_tray: true,
            default_model: String::new(),
            keep_alive: "5m".to_string(),
            allowed_paths: default_allowed_paths(),
            session_outputs_directory: String::new(),
            hardware_accel: "gpu".to_string(),
            multi_model: false,
            show_gpu_status: false,
            compression_enabled: true,
            compression_threshold: 85,
            response_language: String::new(),
            link_preview_enabled: true,
            ollama_setup_skipped: false,
            onboarding_completed: false,
        }
    }
}

impl AdvancedSettings {
    pub fn normalized(mut self) -> Self {
        self.compression_threshold = self.compression_threshold.min(100);
        self.session_outputs_directory =
            normalize_optional_directory(&self.session_outputs_directory).unwrap_or_default();
        self
    }
}

pub(crate) fn normalize_optional_directory(value: &str) -> Option<String> {
    let value = value.trim();
    let path = std::path::Path::new(value);
    if value.is_empty() {
        return Some(String::new());
    }
    if value.len() > 4_096
        || value.chars().any(char::is_control)
        || !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        || !path.is_dir()
    {
        return None;
    }
    path.canonicalize()
        .ok()
        .and_then(|path| path.to_str().map(str::to_string))
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;

fn default_allowed_paths() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        vec!["C:\\".to_string()]
    }
    #[cfg(not(target_os = "windows"))]
    {
        vec!["/".to_string()]
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HeartbeatConfig {
    pub global_paused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledWakeup {
    pub id: String,
    pub name: String,
    pub model: String,
    pub provider: String,
    pub prompt: String,
    pub schedule: WakeupSchedule,
    #[serde(default)]
    pub description: String,
    pub active: bool,
    #[serde(default)]
    pub paused_by_global: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum WakeupSchedule {
    Once { datetime: String },
    Daily { time: String },
    Weekly { weekday: u8, time: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WakeupRunStatus {
    Ok,
    Error,
    Missed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WakeupRun {
    pub wakeup_id: String,
    pub scheduled_for: String,
    pub fired_at: String,
    pub status: WakeupRunStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WakeupStatusSummary {
    pub wakeup_id: String,
    pub next_fire_at: Option<String>,
    pub last_run: Option<WakeupRun>,
}
