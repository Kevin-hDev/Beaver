use super::types::{
    ExtensionApiLevel, ExtensionKind, ExtensionManifest, ExtensionRecord, ExtensionTool,
    ExtensionUiMode, MAX_EVENTS_PER_EXTENSION, MAX_EXTENSIONS, MAX_IDENTIFIER_CHARS,
    MAX_TOOLS_PER_EXTENSION,
};
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Component, Path};

const MAX_NAME_CHARS: usize = 100;
const MAX_TEXT_CHARS: usize = 2_000;
const MAX_PATH_CHARS: usize = 4_096;

pub fn records(records: &[ExtensionRecord]) -> Result<(), String> {
    if records.len() > MAX_EXTENSIONS {
        return Err("Trop d'extensions enregistrées.".to_string());
    }
    let mut identifiers = HashSet::with_capacity(records.len());
    for record in records {
        manifest(&record.manifest)?;
        if !identifiers.insert(record.manifest.id.as_str()) {
            return Err("Identifiant d'extension dupliqué.".to_string());
        }
        if record.kind == ExtensionKind::Local {
            validate_local_source(&record.source)?;
            if record.manifest.runtime != "node" {
                return Err("Runtime d'extension incohérent.".to_string());
            }
        }
        super::origin_validation::record(record)?;
        super::ui_artifact::validate_record(record)?;
        contributions(&record.contributions.tools, &record.contributions.events)?;
    }
    Ok(())
}

pub fn manifest(manifest: &ExtensionManifest) -> Result<(), String> {
    identifier(&manifest.id)?;
    text(&manifest.name, MAX_NAME_CHARS)?;
    text(&manifest.version, 64)?;
    if manifest.beaver_api != super::types::BEAVER_API_VERSION {
        return Err("Version de l'API Beaver incompatible.".to_string());
    }
    if manifest.runtime != "node" && manifest.runtime != "builtin" {
        return Err("Runtime d'extension non pris en charge.".to_string());
    }
    if manifest.access != "full" && manifest.access != "core" {
        return Err("Niveau d'accès d'extension invalide.".to_string());
    }
    if manifest.runtime == "node" && manifest.access != "full" {
        return Err("Une extension Node.js possède un accès complet.".to_string());
    }
    if manifest.kind_requires_main() {
        relative_source_path(
            manifest
                .main
                .as_deref()
                .ok_or_else(|| "Point d'entrée d'extension manquant.".to_string())?,
        )?;
    }
    if let Some(ui) = &manifest.ui {
        if ui.api_version != super::ui_contract::EXTENSION_UI_API_VERSION {
            return Err("Version de l'interface d'extension incompatible.".to_string());
        }
        match (&ui.mode, ui.entry.as_deref()) {
            (ExtensionUiMode::Standard, None) => {}
            (ExtensionUiMode::Advanced, Some(entry))
                if manifest.api_level == ExtensionApiLevel::Advanced =>
            {
                relative_source_path(entry)?;
                let allowed = Path::new(entry)
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| ["js", "mjs", "jsx", "ts", "mts", "tsx"].contains(&value));
                if !allowed {
                    return Err("Point d'entrée UI invalide.".to_string());
                }
            }
            _ => return Err("Déclaration UI d'extension invalide.".to_string()),
        }
    }
    if manifest.ui.is_some() && manifest.ui_legacy.is_some() {
        return Err("Déclaration UI d'extension invalide.".to_string());
    }
    // La chaîne UI v1 reste une donnée de diagnostic inerte : l'interpréter
    // comme un chemin rendrait à tort les outils sains de l'extension invalides.
    if let Some(legacy) = &manifest.ui_legacy {
        source_input(legacy)?;
    }
    for value in [
        manifest.author.as_deref(),
        manifest.homepage.as_deref(),
        manifest.description.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        text(value, MAX_TEXT_CHARS)?;
    }
    Ok(())
}

pub fn contributions(tools: &[ExtensionTool], events: &[String]) -> Result<(), String> {
    if tools.len() > MAX_TOOLS_PER_EXTENSION || events.len() > MAX_EVENTS_PER_EXTENSION {
        return Err("Trop de contributions déclarées.".to_string());
    }
    for tool in tools {
        identifier(&tool.name)?;
        text(&tool.description, MAX_TEXT_CHARS)?;
        validate_schema(&tool.parameters)?;
    }
    for event in events {
        identifier(event)?;
    }
    Ok(())
}

pub fn identifier(value: &str) -> Result<(), String> {
    let valid = !value.is_empty()
        && value.chars().count() <= MAX_IDENTIFIER_CHARS
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
        && value
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphanumeric())
        && value
            .chars()
            .last()
            .is_some_and(|ch| ch.is_ascii_alphanumeric());
    valid
        .then_some(())
        .ok_or_else(|| "Identifiant d'extension invalide.".to_string())
}

pub fn source_input(value: &str) -> Result<(), String> {
    let valid =
        !value.is_empty() && value.chars().count() <= MAX_PATH_CHARS && !value.contains('\0');
    valid
        .then_some(())
        .ok_or_else(|| "Chemin d'extension invalide.".to_string())
}

pub fn message(value: &Value) -> Result<(), String> {
    super::message_validation::validate(value)
}

pub fn request_payload(value: &Value) -> Result<(), String> {
    super::message_validation::validate_request_payload(value)
}

fn relative_source_path(value: &str) -> Result<(), String> {
    source_input(value)?;
    let path = Path::new(value);
    let clean = !path.is_absolute()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)));
    clean
        .then_some(())
        .ok_or_else(|| "Point d'entrée d'extension invalide.".to_string())
}

fn validate_local_source(value: &str) -> Result<(), String> {
    source_input(value)?;
    let path = Path::new(value);
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("Source d'extension invalide.".to_string());
    }
    Ok(())
}

fn text(value: &str, max_chars: usize) -> Result<(), String> {
    (!value.trim().is_empty() && value.chars().count() <= max_chars)
        .then_some(())
        .ok_or_else(|| "Métadonnée d'extension invalide.".to_string())
}

fn validate_schema(value: &Value) -> Result<(), String> {
    message(value)?;
    let object = value
        .as_object()
        .ok_or_else(|| "Schéma d'outil invalide.".to_string())?;
    if object.get("type").and_then(Value::as_str) != Some("object") {
        return Err("Le schéma d'un outil doit décrire un objet.".to_string());
    }
    crate::services::mcp_bridge::arguments::validate_schema(value)
        .map_err(|_| "Schéma d'outil invalide.".to_string())?;
    Ok(())
}

trait ManifestKind {
    fn kind_requires_main(&self) -> bool;
}

impl ManifestKind for ExtensionManifest {
    fn kind_requires_main(&self) -> bool {
        self.runtime != "builtin"
            && matches!(
                self.api_level,
                ExtensionApiLevel::Stable | ExtensionApiLevel::Advanced
            )
    }
}
