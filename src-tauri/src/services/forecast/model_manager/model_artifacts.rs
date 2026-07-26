use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Component, Path};
use std::sync::OnceLock;
use subtle::ConstantTimeEq;

use crate::services::forecast::{catalog, validation};

const RAW_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/forecast-sidecar/model-artifacts.json"
));
const MAX_MANIFEST_BYTES: usize = 128 * 1024;
const MAX_MODELS: usize = 32;
const MAX_ARTIFACTS_PER_MODEL: usize = 8;
const MAX_MODEL_BYTES: u64 = 32 * 1024 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ModelArtifacts {
    pub model_id: String,
    pub repository: String,
    pub revision: String,
    pub artifacts: Vec<ModelArtifact>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct ModelArtifact {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ArtifactManifest {
    schema_version: u32,
    models: Vec<ModelArtifacts>,
}

static MANIFEST: OnceLock<Result<ArtifactManifest, String>> = OnceLock::new();

pub(super) fn model(model_id: &str) -> Result<&'static ModelArtifacts, String> {
    let normalized = if model_id == "tabpfn-ts" {
        "tabpfn-ts-3"
    } else {
        model_id
    };
    manifest()?
        .models
        .iter()
        .find(|entry| entry.model_id == normalized)
        .ok_or_else(|| "Source du modèle Forecast indisponible".to_string())
}

pub(super) fn total_size(model_id: &str) -> Result<u64, String> {
    model(model_id)?
        .artifacts
        .iter()
        .try_fold(0u64, |total, artifact| total.checked_add(artifact.size))
        .filter(|total| *total > 0 && *total <= MAX_MODEL_BYTES)
        .ok_or_else(|| "Source du modèle Forecast invalide".to_string())
}

pub(super) fn sha256_matches(actual: &[u8], expected_hex: &str) -> bool {
    let mut expected = [0u8; 32];
    if actual.len() != expected.len() || hex::decode_to_slice(expected_hex, &mut expected).is_err()
    {
        return false;
    }
    actual.ct_eq(&expected).into()
}

fn manifest() -> Result<&'static ArtifactManifest, String> {
    MANIFEST
        .get_or_init(parse_and_validate)
        .as_ref()
        .map_err(Clone::clone)
}

fn parse_and_validate() -> Result<ArtifactManifest, String> {
    if RAW_MANIFEST.is_empty() || RAW_MANIFEST.len() > MAX_MANIFEST_BYTES {
        return Err("Manifeste Forecast invalide".to_string());
    }
    let parsed: ArtifactManifest = serde_json::from_str(RAW_MANIFEST)
        .map_err(|_| "Manifeste Forecast invalide".to_string())?;
    validate_manifest(&parsed)?;
    Ok(parsed)
}

fn validate_manifest(manifest: &ArtifactManifest) -> Result<(), String> {
    if manifest.schema_version != 1
        || manifest.models.is_empty()
        || manifest.models.len() > MAX_MODELS
    {
        return Err("Manifeste Forecast invalide".to_string());
    }
    let mut model_ids = HashSet::with_capacity(MAX_MODELS);
    for model in &manifest.models {
        validate_model(model, &mut model_ids)?;
    }
    let all_local_models_present = catalog::FORECAST_MODELS
        .iter()
        .filter(|model| !model.is_cloud)
        .all(|model| model_ids.contains(model.id));
    if !all_local_models_present {
        return Err("Manifeste Forecast incomplet".to_string());
    }
    Ok(())
}

fn validate_model(model: &ModelArtifacts, model_ids: &mut HashSet<String>) -> Result<(), String> {
    validation::validate_model_id(&model.model_id)?;
    let catalog_model = catalog::find_model(&model.model_id)
        .filter(|entry| !entry.is_cloud)
        .ok_or_else(|| "Manifeste Forecast invalide".to_string())?;
    if !model_ids.insert(model.model_id.clone())
        || catalog_model.hf_repo != Some(model.repository.as_str())
        || catalog_model.hf_revision != Some(model.revision.as_str())
        || !valid_repository(&model.repository)
        || !valid_hex(&model.revision, 40)
        || model.artifacts.is_empty()
        || model.artifacts.len() > MAX_ARTIFACTS_PER_MODEL
    {
        return Err("Manifeste Forecast invalide".to_string());
    }
    let mut paths = HashSet::with_capacity(MAX_ARTIFACTS_PER_MODEL);
    for artifact in &model.artifacts {
        if artifact.size == 0
            || artifact.size > MAX_MODEL_BYTES
            || !paths.insert(artifact.path.clone())
            || !safe_relative_path(&artifact.path)
            || !valid_hex(&artifact.sha256, 64)
        {
            return Err("Manifeste Forecast invalide".to_string());
        }
    }
    model
        .artifacts
        .iter()
        .try_fold(0u64, |total, item| total.checked_add(item.size))
        .filter(|total| *total > 0 && *total <= MAX_MODEL_BYTES)
        .ok_or_else(|| "Manifeste Forecast invalide".to_string())?;
    Ok(())
}

fn valid_repository(value: &str) -> bool {
    value.len() <= 160
        && value.split('/').count() == 2
        && value.split('/').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        })
}

fn safe_relative_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 240
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'_' | b'.'))
        && !Path::new(value).is_absolute()
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn valid_hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
#[path = "model_artifacts_tests.rs"]
mod tests;
