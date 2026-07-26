use include_dir::{include_dir, Dir};
use std::path::PathBuf;
use uuid::Uuid;

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
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Erreur création dossier traductions : {e}"))?;
    }
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("translation.md");
    let tmp = path.with_file_name(format!(".{}.{}.tmp", Uuid::new_v4(), filename));
    tokio::fs::write(&tmp, text)
        .await
        .map_err(|e| format!("Erreur écriture traduction : {e}"))?;
    tokio::fs::rename(&tmp, &path)
        .await
        .map_err(|e| format!("Erreur rename traduction : {e}"))?;
    Ok(())
}
