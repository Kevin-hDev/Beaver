use crate::services::agent_local::ollama_base_url;
use crate::services::agent_local::types_ollama::{PullProgress, RegistryModel};
use crate::services::brand;
use futures_util::StreamExt;
use reqwest::Client;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::io::StreamReader;
use tokio_util::sync::CancellationToken;
const REGISTRY_URL: &str = "https://ollama.com";

pub async fn search_models(query: &str) -> Result<Vec<RegistryModel>, String> {
    if query.len() > 200 {
        return Err("ollama-query-too-long".into());
    }
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("{REGISTRY_URL}/search?q={}", urlencoded(query));
    let resp = client
        .get(&url)
        .header("User-Agent", brand::user_agent())
        .send()
        .await
        .map_err(|e| format!("Recherche impossible: {e}"))?;

    if !resp.status().is_success() {
        return Err("ollama-registry-error".into());
    }

    let body = resp.bytes().await.map_err(|e| e.to_string())?;
    if body.len() > 2 * 1024 * 1024 {
        return Err("ollama-registry-too-large".into());
    }
    let html = String::from_utf8_lossy(&body);
    Ok(parse_search_html(&html))
}

fn parse_search_html(html: &str) -> Vec<RegistryModel> {
    let mut models = Vec::new();
    // Les résultats sont dans des liens <a href="/library/MODEL_NAME">
    // avec le nom du modèle et une description
    for line in html.lines() {
        let trimmed = line.trim();
        if let Some(start) = trimmed.find("href=\"/library/") {
            let after = &trimmed[start + 15..];
            if let Some(end) = after.find('"') {
                let name = after[..end].to_string();
                if !name.is_empty()
                    && name
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
                    && !models.iter().any(|m: &RegistryModel| m.name == name)
                {
                    models.push(RegistryModel {
                        name,
                        description: String::new(),
                        tags: vec![],
                        is_installed: false,
                    });
                }
            }
        }
    }
    models
}

pub async fn pull_model_with_callback<F>(
    name: &str,
    mut on_progress: F,
    cancel: &CancellationToken,
    pulled_digests: &mut Vec<String>,
) -> Result<(), String>
where
    F: FnMut(PullProgress),
{
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(3600))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .post(format!("{}/api/pull", ollama_base_url()))
        .json(&serde_json::json!({ "model": name, "stream": true }))
        .send()
        .await
        .map_err(|e| format!("Connexion Ollama impossible: {e}"))?;

    let byte_stream = resp
        .bytes_stream()
        .map(|r| r.map_err(std::io::Error::other));
    let mut lines = BufReader::new(StreamReader::new(byte_stream)).lines();
    let digests = pulled_digests;
    let mut line_count: u64 = 0;
    const MAX_PULL_LINES: u64 = 500_000;
    const MAX_STATUS_LEN: usize = 256;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                return Err("cancelled".to_string());
            }
            line = lines.next_line() => {
                let line = line.map_err(|e| e.to_string())?;
                let Some(line) = line else { break };
                line_count += 1;
                if line_count > MAX_PULL_LINES {
                    return Err("ollama-pull-stream-limit".into());
                }
                if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(&line) {
                    let mut status = chunk["status"].as_str().unwrap_or("").to_string();
                    status.truncate(MAX_STATUS_LEN);
                    if let Some(digest) = extract_digest(&status) {
                        if digests.len() < 64 && !digests.contains(&digest) {
                            digests.push(digest);
                        }
                    }
                    let completed = chunk["completed"].as_u64();
                    let total = chunk["total"].as_u64();
                    on_progress(PullProgress {
                        status: status.clone(), completed, total,
                    });
                    if status == "success" { return Ok(()); }
                    if chunk["error"].as_str().is_some() {
                        return Err("ollama-pull-error".into());
                    }
                }
            }
        }
    }
    Ok(())
}

fn extract_digest(status: &str) -> Option<String> {
    let trimmed = status.strip_prefix("pulling ")?;
    let digest = trimmed.trim();
    if digest.len() >= 12 {
        Some(digest.to_string())
    } else {
        None
    }
}

pub fn cleanup_partial_blobs(digests: &[String]) -> usize {
    if digests.is_empty() {
        return 0;
    }

    let blobs_dir = dirs::home_dir()
        .map(|h| h.join(".ollama/models/blobs"))
        .unwrap_or_default();
    if !blobs_dir.is_dir() {
        return 0;
    }

    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(&blobs_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if !name_str.contains("-partial") {
                continue;
            }
            if digests.iter().any(|d| name_str.contains(d))
                && std::fs::remove_file(entry.path()).is_ok()
            {
                count += 1;
            }
        }
    }
    count
}

pub async fn delete_model(name: &str) -> Result<(), String> {
    let client = Client::new();
    let resp = client
        .delete(format!("{}/api/delete", ollama_base_url()))
        .json(&serde_json::json!({ "model": name }))
        .send()
        .await
        .map_err(|e| {
            ::log::error!("[ollama] /api/delete: {e}");
            "ollama-delete-error".to_string()
        })?;

    if !resp.status().is_success() {
        ::log::error!("[ollama] /api/delete failed");
        return Err("ollama-delete-error".into());
    }
    Ok(())
}

fn urlencoded(s: &str) -> String {
    urlencoding::encode(s).into_owned()
}
