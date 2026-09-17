use super::types::{
    ExtensionContributions, ExtensionKind, ExtensionManifest, ExtensionOrigin, ExtensionOriginKind,
    ExtensionRecord, ExtensionStatus, MAX_MESSAGE_BYTES,
};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
enum ManifestFailure {
    Invalid,
    NotBeaverExtension,
    ApiIncompatible,
}

struct ManifestError {
    failure: ManifestFailure,
    detail: String,
}

type ManifestResult<T> = Result<T, ManifestError>;

pub struct LocalExtension {
    pub record: ExtensionRecord,
}

pub fn load_local(input: &str) -> Result<LocalExtension, String> {
    load(input).map_err(|error| error.detail)
}

pub fn load_managed(input: &str) -> Result<LocalExtension, super::OperationFailure> {
    load(input).map_err(|error| match error.failure {
        ManifestFailure::Invalid => super::OperationFailure::ManifestInvalid,
        ManifestFailure::NotBeaverExtension => super::OperationFailure::NotBeaverExtension,
        ManifestFailure::ApiIncompatible => super::OperationFailure::ApiIncompatible,
    })
}

fn load(input: &str) -> ManifestResult<LocalExtension> {
    super::validation::source_input(input).map_err(invalid)?;
    let selected = dunce::canonicalize(PathBuf::from(input))
        .map_err(|_| invalid("Extension locale introuvable."))?;
    let (root, mut manifest) = if selected.is_dir() {
        from_directory(&selected)?
    } else if super::manifest_source::is_source_file(&selected) {
        super::manifest_source::from_source_file(&selected).map_err(invalid)?
    } else {
        from_manifest_file(&selected)?
    };
    if manifest.beaver_api != super::types::BEAVER_API_VERSION {
        return Err(failure(
            ManifestFailure::ApiIncompatible,
            "Version de l'API Beaver incompatible.",
        ));
    }
    super::validation::manifest(&manifest).map_err(invalid)?;
    let main_path = resolve_entry(&root, manifest.main.as_deref())?;
    let relative_main = main_path
        .strip_prefix(&root)
        .ok()
        .and_then(Path::to_str)
        .ok_or_else(|| invalid("Point d'entrée d'extension invalide."))?;
    let source = root
        .to_str()
        .ok_or_else(|| invalid("Source d'extension invalide."))?;
    manifest.main = Some(relative_main.to_string());
    let mut record = ExtensionRecord {
        manifest,
        kind: ExtensionKind::Local,
        source: source.to_string(),
        origin: Some(ExtensionOrigin {
            kind: ExtensionOriginKind::Local,
            locator: source.to_string(),
            revision: None,
        }),
        enabled: false,
        trusted: false,
        fingerprint: None,
        ui_artifact: None,
        trusted_at: None,
        show_in_chat: false,
        status: ExtensionStatus::Inactive,
        last_error: None,
        last_activated_at: None,
        sensitive_access_granted: false,
        contributions: ExtensionContributions::default(),
    };
    if !record
        .manifest
        .ui
        .as_ref()
        .is_some_and(|ui| ui.mode == super::types::ExtensionUiMode::Advanced)
    {
        record.fingerprint = Some(
            super::fingerprint::calculate(&record)
                .map_err(|_| invalid(super::error_codes::FINGERPRINT_FAILED))?,
        );
    }
    Ok(LocalExtension { record })
}

pub fn resolve_record_entry(record: &ExtensionRecord) -> Result<PathBuf, String> {
    let root = dunce::canonicalize(PathBuf::from(&record.source))
        .map_err(|_| "Source d'extension introuvable.".to_string())?;
    resolve_entry(&root, record.manifest.main.as_deref()).map_err(|error| error.detail)
}

fn from_directory(root: &Path) -> ManifestResult<(PathBuf, ExtensionManifest)> {
    for file_name in super::manifest_source::MANIFEST_FILES {
        let path = root.join(file_name);
        if path.is_file() {
            let manifest_path = resolve_inside(root, &path)
                .map_err(|_| invalid("Manifeste d'extension invalide."))?;
            return from_manifest_file(&manifest_path);
        }
    }
    Err(invalid("Manifeste d'extension introuvable."))
}

fn from_manifest_file(path: &Path) -> ManifestResult<(PathBuf, ExtensionManifest)> {
    let root = dunce::canonicalize(
        path.parent()
            .ok_or_else(|| invalid("Manifeste d'extension invalide."))?,
    )
    .map_err(|_| invalid("Source d'extension introuvable."))?;
    let value = read_json(path)?;
    let is_package = path.file_name().and_then(|name| name.to_str()) == Some("package.json");
    let manifest_value = if is_package {
        package_manifest(&value)?
    } else {
        value
    };
    let manifest = serde_json::from_value(manifest_value)
        .map_err(|_| invalid("Manifeste d'extension invalide."))?;
    Ok((root, manifest))
}

fn package_manifest(package: &Value) -> ManifestResult<Value> {
    let package_object = package
        .as_object()
        .ok_or_else(|| invalid("Package d'extension invalide."))?;
    let mut manifest = package_object
        .get("beaver")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| {
            failure(
                ManifestFailure::NotBeaverExtension,
                "Configuration Beaver absente du package.",
            )
        })?;
    copy_package_field(package_object, &mut manifest, "name");
    copy_package_field(package_object, &mut manifest, "version");
    copy_package_field(package_object, &mut manifest, "description");
    if !manifest.contains_key("main") {
        copy_package_field(package_object, &mut manifest, "main");
    }
    Ok(Value::Object(manifest))
}

fn copy_package_field(package: &Map<String, Value>, manifest: &mut Map<String, Value>, key: &str) {
    if !manifest.contains_key(key) {
        if let Some(value) = package.get(key).filter(|value| value.is_string()) {
            manifest.insert(key.to_string(), value.clone());
        }
    }
}

fn resolve_entry(root: &Path, main: Option<&str>) -> ManifestResult<PathBuf> {
    let main = main.ok_or_else(|| invalid("Point d'entrée d'extension manquant."))?;
    let resolved = resolve_inside(root, &root.join(main))
        .map_err(|_| invalid("Point d'entrée d'extension introuvable."))?;
    if !resolved.starts_with(root)
        || !resolved.is_file()
        || !super::manifest_source::is_source_file(&resolved)
    {
        return Err(invalid("Point d'entrée d'extension invalide."));
    }
    Ok(resolved)
}

fn read_json(path: &Path) -> ManifestResult<Value> {
    let metadata =
        std::fs::metadata(path).map_err(|_| invalid("Manifeste d'extension indisponible."))?;
    if metadata.len() > MAX_MESSAGE_BYTES as u64 {
        return Err(invalid("Manifeste d'extension trop volumineux."));
    }
    let bytes = std::fs::read(path).map_err(|_| invalid("Manifeste d'extension indisponible."))?;
    serde_json::from_slice(&bytes).map_err(|_| invalid("Manifeste d'extension invalide."))
}

fn resolve_inside(root: &Path, requested: &Path) -> std::io::Result<PathBuf> {
    // Compare native canonical paths: dunce can strip the Windows prefix from
    // the root but retain it on a child that crosses the long-path boundary.
    let canonical_root = std::fs::canonicalize(root)?;
    let resolved = std::fs::canonicalize(requested)?;
    let relative = resolved
        .strip_prefix(&canonical_root)
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::PermissionDenied))?;
    // Keep the caller's root representation for the persisted relative entry.
    Ok(root.join(relative))
}

fn invalid(detail: impl Into<String>) -> ManifestError {
    failure(ManifestFailure::Invalid, detail)
}

fn failure(failure: ManifestFailure, detail: impl Into<String>) -> ManifestError {
    ManifestError {
        failure,
        detail: detail.into(),
    }
}

#[cfg(test)]
#[path = "manifest_path_tests.rs"]
mod path_tests;
