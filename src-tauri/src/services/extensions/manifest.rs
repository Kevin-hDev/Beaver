use super::types::{
    ExtensionApiLevel, ExtensionContributions, ExtensionKind, ExtensionManifest, ExtensionRecord,
    ExtensionStatus, BEAVER_API_VERSION, MAX_MESSAGE_BYTES,
};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

const MANIFEST_FILES: &[&str] = &["beaver-extension.json", "beaver.json", "package.json"];
const SOURCE_EXTENSIONS: &[&str] = &[
    "js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx", "mtsx", "ctsx",
];

pub struct LocalExtension {
    pub record: ExtensionRecord,
}

pub fn load_local(input: &str) -> Result<LocalExtension, String> {
    super::validation::source_input(input)?;
    let selected = PathBuf::from(input)
        .canonicalize()
        .map_err(|_| "Extension locale introuvable.".to_string())?;
    let (root, mut manifest) = if selected.is_dir() {
        from_directory(&selected)?
    } else if is_source_file(&selected) {
        from_source_file(&selected)?
    } else {
        from_manifest_file(&selected)?
    };
    super::validation::manifest(&manifest)?;
    let main_path = resolve_entry(&root, manifest.main.as_deref())?;
    let relative_main = main_path
        .strip_prefix(&root)
        .ok()
        .and_then(Path::to_str)
        .ok_or_else(|| "Point d'entrée d'extension invalide.".to_string())?;
    let source = root
        .to_str()
        .ok_or_else(|| "Source d'extension invalide.".to_string())?;
    manifest.main = Some(relative_main.to_string());
    let record = ExtensionRecord {
        manifest,
        kind: ExtensionKind::Local,
        source: source.to_string(),
        enabled: false,
        trusted: false,
        show_in_chat: false,
        status: ExtensionStatus::Inactive,
        last_error: None,
        last_activated_at: None,
        contributions: ExtensionContributions::default(),
    };
    Ok(LocalExtension { record })
}

pub fn resolve_record_entry(record: &ExtensionRecord) -> Result<PathBuf, String> {
    let root = PathBuf::from(&record.source)
        .canonicalize()
        .map_err(|_| "Source d'extension introuvable.".to_string())?;
    resolve_entry(&root, record.manifest.main.as_deref())
}

fn from_directory(root: &Path) -> Result<(PathBuf, ExtensionManifest), String> {
    for file_name in MANIFEST_FILES {
        let path = root.join(file_name);
        if path.is_file() {
            let manifest_path = path
                .canonicalize()
                .map_err(|_| "Manifeste d'extension invalide.".to_string())?;
            if !manifest_path.starts_with(root) {
                return Err("Manifeste d'extension invalide.".to_string());
            }
            return from_manifest_file(&manifest_path);
        }
    }
    Err("Manifeste d'extension introuvable.".to_string())
}

fn from_manifest_file(path: &Path) -> Result<(PathBuf, ExtensionManifest), String> {
    let root = path
        .parent()
        .ok_or_else(|| "Manifeste d'extension invalide.".to_string())?
        .canonicalize()
        .map_err(|_| "Source d'extension introuvable.".to_string())?;
    let value = read_json(path)?;
    let is_package = path.file_name().and_then(|name| name.to_str()) == Some("package.json");
    let manifest_value = if is_package {
        package_manifest(&value)?
    } else {
        value
    };
    let manifest = serde_json::from_value(manifest_value)
        .map_err(|_| "Manifeste d'extension invalide.".to_string())?;
    Ok((root, manifest))
}

fn package_manifest(package: &Value) -> Result<Value, String> {
    let package_object = package
        .as_object()
        .ok_or_else(|| "Package d'extension invalide.".to_string())?;
    let mut manifest = package_object
        .get("beaver")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| "Configuration Beaver absente du package.".to_string())?;
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

fn from_source_file(path: &Path) -> Result<(PathBuf, ExtensionManifest), String> {
    let root = path
        .parent()
        .ok_or_else(|| "Source d'extension invalide.".to_string())?
        .canonicalize()
        .map_err(|_| "Source d'extension introuvable.".to_string())?;
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("extension");
    let slug: String = stem
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
        .take(40)
        .collect();
    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let main = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Source d'extension invalide.".to_string())?;
    Ok((
        root,
        ExtensionManifest {
            id: format!(
                "local.{}.{}",
                if slug.is_empty() { "extension" } else { &slug },
                &suffix[..8]
            ),
            name: if stem.is_empty() {
                "Extension".to_string()
            } else {
                stem.to_string()
            },
            version: "0.0.0".to_string(),
            beaver_api: BEAVER_API_VERSION.to_string(),
            runtime: "node".to_string(),
            main: Some(main.to_string()),
            ui: None,
            access: "full".to_string(),
            api_level: ExtensionApiLevel::Advanced,
            author: None,
            homepage: None,
            description: None,
        },
    ))
}

fn resolve_entry(root: &Path, main: Option<&str>) -> Result<PathBuf, String> {
    let main = main.ok_or_else(|| "Point d'entrée d'extension manquant.".to_string())?;
    let resolved = root
        .join(main)
        .canonicalize()
        .map_err(|_| "Point d'entrée d'extension introuvable.".to_string())?;
    if !resolved.starts_with(root) || !resolved.is_file() || !is_source_file(&resolved) {
        return Err("Point d'entrée d'extension invalide.".to_string());
    }
    Ok(resolved)
}

fn read_json(path: &Path) -> Result<Value, String> {
    let metadata =
        std::fs::metadata(path).map_err(|_| "Manifeste d'extension indisponible.".to_string())?;
    if metadata.len() > MAX_MESSAGE_BYTES as u64 {
        return Err("Manifeste d'extension trop volumineux.".to_string());
    }
    let bytes =
        std::fs::read(path).map_err(|_| "Manifeste d'extension indisponible.".to_string())?;
    serde_json::from_slice(&bytes).map_err(|_| "Manifeste d'extension invalide.".to_string())
}

fn is_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| SOURCE_EXTENSIONS.contains(&value.to_ascii_lowercase().as_str()))
}
