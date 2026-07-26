use serde::Serialize;

use crate::services::{brand, ollama_port};

const OLLAMA_GITHUB_REPO: &str = "ollama/ollama";

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

pub async fn fetch_latest_github_version() -> Result<(String, String), String> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        OLLAMA_GITHUB_REPO
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|_| "ollama-github-error".to_string())?;

    let resp = client
        .get(&url)
        .header("User-Agent", brand::user_agent())
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|_| "ollama-github-error".to_string())?;

    if !resp.status().is_success() {
        return Err("ollama-github-error".into());
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|_| "ollama-github-error".to_string())?;

    let tag = json["tag_name"]
        .as_str()
        .unwrap_or_default()
        .trim_start_matches('v');

    if tag.is_empty() {
        return Err("no tag_name in release".into());
    }

    let download_url = find_ollama_asset(&json).unwrap_or_default();

    Ok((tag.to_string(), download_url))
}

fn find_ollama_asset(json: &serde_json::Value) -> Option<String> {
    let assets = json["assets"].as_array()?;
    let name_pattern = ollama_archive_name();

    assets.iter().find_map(|a| {
        let name = a["name"].as_str().unwrap_or_default();
        if name == name_pattern {
            a["browser_download_url"].as_str().map(|s| s.to_string())
        } else {
            None
        }
    })
}

fn ollama_archive_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "ollama-darwin.tgz"
    } else if cfg!(target_os = "windows") {
        "ollama-windows-amd64.zip"
    } else {
        "ollama-linux-amd64.tar.zst"
    }
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

    let (latest, _url) = match fetch_latest_github_version().await {
        Ok(v) => v,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ollama_archive_name_returns_platform_specific() {
        let name = ollama_archive_name();
        assert!(!name.is_empty());
        if cfg!(target_os = "macos") {
            assert_eq!(name, "ollama-darwin.tgz");
        } else if cfg!(target_os = "windows") {
            assert_eq!(name, "ollama-windows-amd64.zip");
        } else {
            assert_eq!(name, "ollama-linux-amd64.tar.zst");
        }
    }

    #[test]
    fn find_ollama_asset_extracts_url() {
        let json = serde_json::json!({
            "assets": [
                {
                    "name": "ollama-darwin.tgz",
                    "browser_download_url": "https://github.com/ollama/ollama/releases/download/v0.23.1/ollama-darwin.tgz"
                },
                {
                    "name": "ollama-linux-amd64.tar.zst",
                    "browser_download_url": "https://github.com/ollama/ollama/releases/download/v0.23.1/ollama-linux-amd64.tar.zst"
                }
            ]
        });
        let url = find_ollama_asset(&json);
        assert!(url.is_some());
        assert!(url.unwrap().contains("ollama"));
    }

    #[test]
    fn find_ollama_asset_returns_none_on_empty() {
        let json = serde_json::json!({ "assets": [] });
        assert!(find_ollama_asset(&json).is_none());
    }
}
