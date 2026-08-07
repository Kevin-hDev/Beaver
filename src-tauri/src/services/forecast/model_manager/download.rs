use crate::services::{
    brand,
    model_downloads::{ModelDownloadPhase, ProgressUpdate},
};
use reqwest::redirect::Policy;
use std::path::Path;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

use super::model_artifacts::{self, ModelArtifact};

#[path = "download_io.rs"]
mod download_io;

const MIN_PROGRESS_STEP: f64 = 1.0;
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);

pub async fn download_model(
    model_id: &str,
    target_dir: &Path,
    cancel: &CancellationToken,
    on_progress: &(dyn Fn(ProgressUpdate) + Send + Sync),
) -> Result<(), String> {
    let model = model_artifacts::model(model_id)?;
    let total_size = model_artifacts::total_size(model_id)?;
    let client = download_client()?;
    let mut downloaded = 0u64;
    let mut last_percent_sent = 0.0;

    for artifact in &model.artifacts {
        let url = artifact_url(&model.repository, &model.revision, &artifact.path)?;
        let destination = target_dir.join(&artifact.path);
        if let Some(parent) = destination.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|_| "Préparation du téléchargement impossible".to_string())?;
        }
        download_single(
            &client,
            &url,
            &destination,
            artifact,
            &mut downloaded,
            total_size,
            cancel,
            on_progress,
            &mut last_percent_sent,
        )
        .await?;
    }

    on_progress(ProgressUpdate {
        phase: ModelDownloadPhase::Downloading,
        downloaded: total_size,
        total: total_size,
        percent: 99,
    });
    Ok(())
}

fn download_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .redirect(Policy::custom(|attempt| {
            if attempt.previous().len() >= 5 || !allowed_download_url(attempt.url()) {
                attempt.stop()
            } else {
                attempt.follow()
            }
        }))
        .build()
        .map_err(|_| "Téléchargement du modèle impossible".to_string())
}

fn artifact_url(repository: &str, revision: &str, path: &str) -> Result<String, String> {
    let url = format!("https://huggingface.co/{repository}/resolve/{revision}/{path}");
    let parsed =
        reqwest::Url::parse(&url).map_err(|_| "Source du modèle Forecast invalide".to_string())?;
    if !allowed_download_url(&parsed) || parsed.host_str() != Some("huggingface.co") {
        return Err("Source du modèle Forecast invalide".to_string());
    }
    Ok(url)
}

fn allowed_download_url(url: &reqwest::Url) -> bool {
    if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    url.host_str().is_some_and(|host| {
        host == "huggingface.co"
            || host.ends_with(".huggingface.co")
            || host == "hf.co"
            || host.ends_with(".hf.co")
    })
}

#[expect(
    clippy::too_many_arguments,
    reason = "boundary parameters remain explicit and locally audited"
)]
async fn download_single(
    client: &reqwest::Client,
    url: &str,
    destination: &Path,
    artifact: &ModelArtifact,
    downloaded: &mut u64,
    total_size: u64,
    cancel: &CancellationToken,
    on_progress: &(dyn Fn(ProgressUpdate) + Send + Sync),
    last_percent_sent: &mut f64,
) -> Result<(), String> {
    let response = tokio::time::timeout(
        RESPONSE_TIMEOUT,
        client
            .get(url)
            .header("User-Agent", brand::user_agent())
            .send(),
    )
    .await
    .map_err(|_| "Téléchargement du modèle expiré".to_string())?
    .map_err(|_| "Téléchargement du modèle impossible".to_string())?;
    if !response.status().is_success() || !allowed_download_url(response.url()) {
        return Err("Téléchargement du modèle impossible".to_string());
    }
    if response
        .content_length()
        .is_some_and(|size| size > artifact.size)
    {
        return Err("Téléchargement du modèle invalide".to_string());
    }
    download_io::write_model_response(
        response,
        destination,
        artifact,
        downloaded,
        total_size,
        cancel,
        on_progress,
        last_percent_sent,
        MIN_PROGRESS_STEP,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::{allowed_download_url, artifact_url};

    #[test]
    fn only_approved_hugging_face_hosts_are_downloaded() {
        let official = reqwest::Url::parse("https://cas-bridge.xethub.hf.co/file").unwrap();
        let spoofed = reqwest::Url::parse("https://hf.co.evil.invalid/file").unwrap();
        assert!(allowed_download_url(&official));
        assert!(!allowed_download_url(&spoofed));
    }

    #[test]
    fn artifact_url_keeps_the_pinned_revision() {
        let url = artifact_url("org/model", &"a".repeat(40), "model.safetensors").unwrap();
        assert!(url.contains(&format!("/resolve/{}/", "a".repeat(40))));
    }
}
