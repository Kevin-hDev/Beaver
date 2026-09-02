use super::types::{ExtensionApiLevel, ExtensionManifest, BEAVER_API_VERSION};
use std::path::{Path, PathBuf};

pub(super) const MANIFEST_FILES: &[&str] =
    &["beaver-extension.json", "beaver.json", "package.json"];

const SOURCE_EXTENSIONS: &[&str] = &[
    "js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx", "mtsx", "ctsx",
];

pub fn from_source_file(path: &Path) -> Result<(PathBuf, ExtensionManifest), String> {
    let root = dunce::canonicalize(
        path.parent()
            .ok_or_else(|| "Source d'extension invalide.".to_string())?,
    )
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
            essential: false,
            author: None,
            homepage: None,
            description: None,
        },
    ))
}

pub fn is_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| SOURCE_EXTENSIONS.contains(&value.to_ascii_lowercase().as_str()))
}

pub fn manifest_path(root: &Path) -> Option<PathBuf> {
    MANIFEST_FILES
        .iter()
        .map(|name| root.join(name))
        .find(|path| path.is_file())
}
