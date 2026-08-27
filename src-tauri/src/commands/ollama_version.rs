use serde::Serialize;

use crate::services::agent_local::ollama_client::OllamaClient;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaBinaryUpdate {
    pub current_version: String,
    pub latest_version: String,
}

pub async fn fetch_installed_version(ollama: &OllamaClient) -> Result<String, String> {
    let url = format!("{}/api/version", ollama.base_url().await?);
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
pub async fn check_ollama_binary_update(
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<Option<OllamaBinaryUpdate>, String> {
    let current = match fetch_installed_version(&ollama).await {
        Ok(v) => v,
        Err(_) => match ollama.manager().installed_version().await {
            Some(v) => v.to_string(),
            None => return Ok(None),
        },
    };

    let latest =
        crate::services::ollama_manager::release_source::fetch_latest_version_for_update_check()
            .await;
    binary_update_from_versions(current, latest)
}

fn binary_update_from_versions(
    current: String,
    latest: Result<
        crate::services::ollama_manager::OllamaVersion,
        crate::services::ollama_manager::OllamaErrorCode,
    >,
) -> Result<Option<OllamaBinaryUpdate>, String> {
    let latest = latest.map_err(|code| {
        log::warn!(
            "[ollama-update-check] stage=resolve-latest code={}",
            code.as_str()
        );
        "ollama-update-check-failed".to_string()
    })?;
    let latest = latest.to_string();
    if !super::app_update::version_gt(&latest, &current) {
        return Ok(None);
    }

    Ok(Some(OllamaBinaryUpdate {
        current_version: current,
        latest_version: latest,
    }))
}

#[tauri::command]
pub async fn get_ollama_installed_version(
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<Option<String>, String> {
    if let Ok(version) = fetch_installed_version(&ollama).await {
        return Ok(Some(version));
    }
    Ok(ollama
        .manager()
        .installed_version()
        .await
        .map(|version| version.to_string()))
}

#[cfg(test)]
#[path = "ollama_version_tests.rs"]
mod tests;
