use std::{collections::HashSet, path::Path};

use reqwest::Url;

use super::{VoiceCatalog, VoiceCatalogEntry, VoiceEngine, VoiceLanguageMode, VoiceModelRole};
use crate::services::voice::{errors::VoiceError, limits};

const CATALOG_VERSION: u8 = 1;
const ALLOWED_SOURCE_HOSTS: [&str; 3] = ["github.com", "huggingface.co", "api.github.com"];

pub(super) fn validate_catalog(catalog: &VoiceCatalog) -> Result<(), VoiceError> {
    if catalog.version != CATALOG_VERSION
        || catalog.entries.is_empty()
        || catalog.entries.len() > limits::MAX_CATALOG_ENTRIES
    {
        return invalid();
    }
    let mut ids = HashSet::with_capacity(catalog.entries.len());
    for entry in &catalog.entries {
        if !ids.insert(entry.id.as_str()) || validate_entry(entry).is_err() {
            return invalid();
        }
    }
    Ok(())
}

fn validate_entry(entry: &VoiceCatalogEntry) -> Result<(), VoiceError> {
    validate_id(&entry.id)?;
    validate_revision(&entry.revision)?;
    if !super::transfer_http::production_url_allowed(&entry.archive.url) {
        return invalid();
    }
    validate_source_url(&entry.manifest_url)?;
    validate_source_url(&entry.license.url)?;
    validate_sha(&entry.archive.sha256)?;
    if (is_commit_revision(&entry.revision) && !entry.manifest_url.contains(&entry.revision))
        || entry.archive.bytes == 0
        || entry.files.is_empty()
        || entry.files.len() > limits::MAX_CATALOG_FILES_PER_ENTRY
        || entry.languages.len() > limits::MAX_CATALOG_LANGUAGES_PER_ENTRY
        || entry.dialects.len() > limits::MAX_CATALOG_DIALECTS_PER_ENTRY
    {
        return invalid();
    }

    let mut paths = HashSet::with_capacity(entry.files.len());
    let mut installed_bytes = 0_u64;
    for file in &entry.files {
        validate_path(&file.path)?;
        validate_sha(&file.sha256)?;
        if file.bytes == 0 || !paths.insert(file.path.as_str()) {
            return invalid();
        }
        installed_bytes = installed_bytes.checked_add(file.bytes).ok_or_else(error)?;
    }
    if installed_bytes != entry.installed_bytes || !required_files_present(entry, &paths) {
        return invalid();
    }

    let mut languages = HashSet::with_capacity(entry.languages.len());
    if entry
        .languages
        .iter()
        .any(|language| !valid_language(language) || !languages.insert(language.as_str()))
    {
        return invalid();
    }
    let mut dialects = HashSet::with_capacity(entry.dialects.len());
    if entry
        .dialects
        .iter()
        .any(|dialect| !valid_language(dialect) || !dialects.insert(dialect.as_str()))
    {
        return invalid();
    }
    let capability_matches = match entry.engine {
        VoiceEngine::NemoTransducer => {
            entry.role == VoiceModelRole::Asr
                && entry.language_mode == VoiceLanguageMode::AutomaticOnly
                && entry.dialects.is_empty()
        }
        VoiceEngine::CohereTranscribe => {
            entry.role == VoiceModelRole::Asr
                && entry.language_mode == VoiceLanguageMode::ExplicitOnly
                && entry.dialects.is_empty()
        }
        VoiceEngine::Qwen3Asr => {
            entry.role == VoiceModelRole::Asr
                && entry.language_mode == VoiceLanguageMode::AutomaticOnly
        }
        VoiceEngine::SileroVad => entry.role == VoiceModelRole::Vad && entry.dialects.is_empty(),
    };
    if !capability_matches
        || !matches!(
            entry.license.spdx.as_str(),
            "Apache-2.0" | "CC-BY-4.0" | "MIT"
        )
    {
        return invalid();
    }
    match entry.role {
        VoiceModelRole::Asr if entry.languages.is_empty() => invalid(),
        VoiceModelRole::Vad
            if !entry.languages.is_empty()
                || entry.language_mode != VoiceLanguageMode::AutomaticOnly =>
        {
            invalid()
        }
        _ => Ok(()),
    }
}

fn required_files_present(entry: &VoiceCatalogEntry, files: &HashSet<&str>) -> bool {
    let required: &[&str] = match entry.engine {
        VoiceEngine::NemoTransducer => &[
            "encoder.int8.onnx",
            "decoder.int8.onnx",
            "joiner.int8.onnx",
            "tokens.txt",
        ],
        VoiceEngine::CohereTranscribe => &[
            "encoder.int8.onnx",
            "encoder.int8.onnx.data",
            "decoder.int8.onnx",
            "tokens.txt",
        ],
        VoiceEngine::Qwen3Asr => &[
            "conv_frontend.onnx",
            "encoder.int8.onnx",
            "decoder.int8.onnx",
            "tokenizer/merges.txt",
            "tokenizer/tokenizer_config.json",
            "tokenizer/vocab.json",
        ],
        VoiceEngine::SileroVad => &["silero_vad.onnx"],
    };
    required.iter().all(|path| files.contains(path))
}

fn validate_id(value: &str) -> Result<(), VoiceError> {
    if value.is_empty()
        || value.chars().count() > limits::MAX_CATALOG_ID_CHARS
        || !value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
    {
        return invalid();
    }
    Ok(())
}

fn validate_revision(value: &str) -> Result<(), VoiceError> {
    if revision_directory_name(value).is_none() {
        return invalid();
    }
    Ok(())
}

pub(super) fn revision_directory_name(value: &str) -> Option<&str> {
    if is_commit_revision(value) {
        return Some(value);
    }
    value.strip_prefix("sha256:").filter(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn is_commit_revision(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn validate_source_url(value: &str) -> Result<(), VoiceError> {
    if value.chars().count() > limits::MAX_CATALOG_URL_CHARS {
        return invalid();
    }
    let url = Url::parse(value).map_err(|_| error())?;
    if url.scheme() != "https"
        || url.username() != ""
        || url.password().is_some()
        || !ALLOWED_SOURCE_HOSTS.contains(&url.host_str().unwrap_or_default())
    {
        return invalid();
    }
    Ok(())
}

fn validate_path(value: &str) -> Result<(), VoiceError> {
    if value.is_empty()
        || value.chars().count() > limits::MAX_CATALOG_PATH_CHARS
        || value.contains('\\')
        || Path::new(value).is_absolute()
        || Path::new(value)
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return invalid();
    }
    Ok(())
}

fn validate_sha(value: &str) -> Result<(), VoiceError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return invalid();
    }
    Ok(())
}

fn valid_language(value: &str) -> bool {
    (2..=limits::MAX_LANGUAGE_CODE_CHARS).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
        && !value.starts_with('-')
        && !value.ends_with('-')
        && !value.contains("--")
}

fn invalid<T>() -> Result<T, VoiceError> {
    Err(error())
}

fn error() -> VoiceError {
    VoiceError::configuration_unavailable()
}
