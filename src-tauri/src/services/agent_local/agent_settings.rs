use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const LEGACY_FORECAST_TOOLS: &[&str] = &[
    "forecast_data_audit",
    "forecast",
    "forecast_models",
    "forecast_analyze",
    "forecast_read",
];
const TOOL_CATALOG_SCHEMA: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSettings {
    #[serde(default = "default_permission_mode")]
    pub permission_mode: String,
    #[serde(default = "super::tool_catalog::default_enabled_optional_tools")]
    pub enabled_optional_tools: Vec<String>,
    #[serde(default)]
    pub tool_catalog_schema: u32,
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self {
            permission_mode: default_permission_mode(),
            enabled_optional_tools: super::tool_catalog::default_enabled_optional_tools(),
            tool_catalog_schema: TOOL_CATALOG_SCHEMA,
        }
    }
}

impl AgentSettings {
    pub fn normalized(mut self) -> Self {
        if !is_valid_permission_mode(&self.permission_mode) {
            self.permission_mode = default_permission_mode();
        }
        let migrate_forecast = self.tool_catalog_schema < TOOL_CATALOG_SCHEMA
            && LEGACY_FORECAST_TOOLS.iter().all(|tool| {
                self.enabled_optional_tools
                    .iter()
                    .any(|enabled| enabled == tool)
            });
        if migrate_forecast {
            self.enabled_optional_tools.push("forecast_backtest".into());
            self.enabled_optional_tools
                .push("forecast_compare_models".into());
        }
        self.enabled_optional_tools =
            super::tool_catalog::normalize_enabled_optional_tools(&self.enabled_optional_tools);
        self.tool_catalog_schema = TOOL_CATALOG_SCHEMA;
        self
    }
}

fn default_permission_mode() -> String {
    "auto".to_string()
}

fn settings_path() -> PathBuf {
    crate::services::paths::data_dir().join("agent-settings.json")
}

pub async fn load() -> AgentSettings {
    let path = settings_path();
    if !path.exists() {
        return AgentSettings::default();
    }
    match tokio::fs::read_to_string(&path).await {
        Ok(data) => serde_json::from_str::<AgentSettings>(&data)
            .map(AgentSettings::normalized)
            .unwrap_or_default(),
        Err(_) => AgentSettings::default(),
    }
}

pub async fn save(settings: &AgentSettings) -> Result<(), String> {
    if !is_valid_permission_mode(&settings.permission_mode) {
        return Err("permission_mode invalide".into());
    }
    let settings = settings.clone().normalized();
    let path = settings_path();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("tmp");
    let data = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    tokio::fs::write(&tmp, &data)
        .await
        .map_err(|e| e.to_string())?;
    tokio::fs::rename(&tmp, &path)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn get_permission_mode() -> String {
    load().await.permission_mode
}

pub fn with_permission_mode(
    mut settings: AgentSettings,
    mode: String,
) -> Result<AgentSettings, String> {
    if !is_valid_permission_mode(&mode) {
        return Err("permission_mode invalide".into());
    }
    settings.permission_mode = mode;
    Ok(settings.normalized())
}

pub fn with_tool_group_enabled(
    mut settings: AgentSettings,
    group_id: &str,
    enabled: bool,
) -> Result<AgentSettings, String> {
    let tool_ids = super::tool_group_catalog::optional_group_tool_ids(group_id)?;
    if enabled {
        for tool_id in tool_ids {
            if !settings
                .enabled_optional_tools
                .iter()
                .any(|id| id == tool_id)
            {
                settings.enabled_optional_tools.push((*tool_id).to_string());
            }
        }
    } else {
        settings
            .enabled_optional_tools
            .retain(|id| !tool_ids.contains(&id.as_str()));
    }
    Ok(settings.normalized())
}

pub async fn set_optional_tool_enabled(
    tool_id: String,
    enabled: bool,
) -> Result<AgentSettings, String> {
    super::tool_catalog::validate_optional_tool_id(&tool_id)?;
    let mut settings = load().await;
    if enabled {
        if !settings
            .enabled_optional_tools
            .iter()
            .any(|id| id == &tool_id)
        {
            settings.enabled_optional_tools.push(tool_id);
        }
    } else {
        settings.enabled_optional_tools.retain(|id| id != &tool_id);
    }
    let settings = settings.normalized();
    save(&settings).await?;
    Ok(settings)
}

pub async fn set_tool_group_enabled(
    group_id: String,
    enabled: bool,
) -> Result<AgentSettings, String> {
    let settings = with_tool_group_enabled(load().await, &group_id, enabled)?;
    save(&settings).await?;
    Ok(settings)
}

pub async fn is_tool_enabled(tool_id: &str) -> bool {
    let settings = load().await;
    super::tool_catalog::is_enabled(tool_id, &settings.enabled_optional_tools)
}

fn is_valid_permission_mode(mode: &str) -> bool {
    matches!(mode, "auto" | "manual" | "chat")
}

#[cfg(test)]
#[path = "agent_settings_tests.rs"]
mod tests;
