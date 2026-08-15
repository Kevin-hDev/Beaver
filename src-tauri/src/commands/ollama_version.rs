use serde::Serialize;

use crate::services::ollama_port;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaBinaryUpdate {
    pub current_version: String,
    pub latest_version: String,
}

pub async fn fetch_installed_version() -> Result<String, String> {
    let url = format!("{}/api/version", ollama_port::base_url());
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|_| "ollama-version-error".to_string())?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|_| "ollama-not-running".to_string())?;

    if !resp.status().is_success() {
        return Err("ollama-api-error".into());
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|_| "ollama-version-error".to_string())?;

    json["version"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "ollama-version-error".into())
}

#[tauri::command]
pub async fn check_ollama_binary_update() -> Result<Option<OllamaBinaryUpdate>, String> {
    let current = match fetch_installed_version().await {
        Ok(v) => v,
        Err(_) => match super::ollama_bundle_utils::read_version_file() {
            Some(v) => v,
            None => return Ok(None),
        },
    };

    let latest = match crate::services::ollama_manager::release_source::fetch_latest_version().await
    {
        Ok(version) => version.to_string(),
        Err(_) => return Ok(None),
    };

    if !super::app_update::version_gt(&latest, &current) {
        return Ok(None);
    }

    Ok(Some(OllamaBinaryUpdate {
        current_version: current,
        latest_version: latest,
    }))
}
