use super::NativePromptState;
use serde::Deserialize;
use std::time::Duration;

const REGISTRY_BASE: &str = "https://registry.ollama.ai/v2";
const SYSTEM_LAYER: &str = "application/vnd.ollama.image.system";
const MAX_MANIFEST_BYTES: usize = 256 * 1024;
const MAX_MANIFEST_LAYERS: usize = 64;
const MAX_NATIVE_PROMPT_BYTES: u64 = 64 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub enum NativeLayer {
    Absent,
    Present { digest: String, size: u64 },
}

#[derive(Deserialize)]
struct RegistryManifest {
    layers: Vec<RegistryLayer>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegistryLayer {
    media_type: String,
    digest: String,
    size: u64,
}

pub async fn fetch(model: &str) -> Result<NativePromptState, String> {
    let (repository, tag) = registry_model_path(model)
        .ok_or_else(|| "ollama-registry-error".to_string())?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|_| "ollama-registry-error".to_string())?;
    let manifest = client
        .get(format!("{REGISTRY_BASE}/{repository}/manifests/{tag}"))
        .send()
        .await
        .map_err(|_| "ollama-registry-error".to_string())?;
    if !manifest.status().is_success() {
        return Err("ollama-registry-error".into());
    }
    let body = crate::services::secure_http::read_bounded(manifest, MAX_MANIFEST_BYTES)
        .await
        .map_err(|_| "ollama-registry-error".to_string())?;
    let NativeLayer::Present { digest, size } = parse_native_layer(&body)? else {
        return Ok(NativePromptState::Absent);
    };
    let blob = client
        .get(format!("{REGISTRY_BASE}/{repository}/blobs/{digest}"))
        .send()
        .await
        .map_err(|_| "ollama-registry-error".to_string())?;
    if !blob.status().is_success() || blob.content_length().is_some_and(|length| length != size) {
        return Err("ollama-registry-error".into());
    }
    let content = crate::services::secure_http::read_bounded(blob, MAX_NATIVE_PROMPT_BYTES as usize)
        .await
        .map_err(|_| "ollama-registry-error".to_string())?;
    let prompt = String::from_utf8(content.to_vec())
        .map_err(|_| "ollama-registry-error".to_string())?;
    if prompt.trim().is_empty() {
        Ok(NativePromptState::Absent)
    } else {
        Ok(NativePromptState::Present(prompt))
    }
}

pub fn parse_native_layer(bytes: &[u8]) -> Result<NativeLayer, String> {
    let manifest: RegistryManifest =
        serde_json::from_slice(bytes).map_err(|_| "ollama-registry-error".to_string())?;
    if manifest.layers.len() > MAX_MANIFEST_LAYERS {
        return Err("ollama-registry-error".into());
    }
    let Some(layer) = manifest
        .layers
        .into_iter()
        .find(|layer| layer.media_type == SYSTEM_LAYER)
    else {
        return Ok(NativeLayer::Absent);
    };
    if layer.size > MAX_NATIVE_PROMPT_BYTES || !valid_digest(&layer.digest) {
        return Err("ollama-registry-error".into());
    }
    Ok(NativeLayer::Present { digest: layer.digest, size: layer.size })
}

pub fn registry_model_path(name: &str) -> Option<(String, String)> {
    let normalized = name
        .strip_prefix("registry.ollama.ai/")
        .or_else(|| name.strip_prefix("ollama.com/"))
        .unwrap_or(name);
    if normalized.matches('/').count() > 1 {
        return None;
    }
    let (repository, tag) = normalized.rsplit_once(':').unwrap_or((normalized, "latest"));
    if repository.is_empty() || tag.is_empty() || !valid_component(tag) {
        return None;
    }
    let path = if repository.contains('/') {
        repository.to_string()
    } else {
        format!("library/{repository}")
    };
    path.split('/').all(valid_component).then_some((path, tag.into()))
}

fn valid_component(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        })
}

fn valid_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64 && hex.chars().all(|character| character.is_ascii_hexdigit())
    })
}
