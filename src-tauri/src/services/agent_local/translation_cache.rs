use include_dir::{include_dir, Dir};
use std::path::PathBuf;

// Traductions pré-bundlées embarquées dans le binaire au build.
// Permet de livrer l'app avec des traductions pour les modèles populaires.
static BUNDLED: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../translations");

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn translations_dir() -> PathBuf {
    crate::services::paths::data_dir().join("translations")
}

fn filename(model: &str, lang: &str) -> String {
    format!("{}.{}.md", sanitize(model), sanitize(lang))
}

fn translation_path(model: &str, lang: &str) -> PathBuf {
    translations_dir().join(filename(model, lang))
}

fn get_bundled(model: &str, lang: &str) -> Option<String> {
    BUNDLED
        .get_file(filename(model, lang))
        .and_then(|f| f.contents_utf8())
        .map(String::from)
}

pub async fn get_cached(model: &str, lang: &str) -> Option<String> {
    // 1. Override utilisateur (~/.local/share/cl-go-dash/translations/)
    let path = translation_path(model, lang);
    if let Ok(text) = tokio::fs::read_to_string(&path).await {
        return Some(text);
    }
    // 2. Fichier pré-bundlé dans le binaire
    get_bundled(model, lang)
}

pub async fn set_cached(model: &str, lang: &str, text: &str) -> Result<(), String> {
    let path = translation_path(model, lang);
    crate::services::private_store::atomic_write_async(path, text.as_bytes().to_vec())
        .await
        .map_err(|_| "Cache de traduction indisponible".to_string())
}
