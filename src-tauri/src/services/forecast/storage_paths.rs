use regex::Regex;
use std::path::PathBuf;
use std::sync::LazyLock;

static ANALYSIS_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-f0-9\-]+$").unwrap());

pub fn validate_analysis_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 || !ANALYSIS_ID_REGEX.is_match(id) {
        return Err("Identifiant d'analyse invalide".into());
    }
    Ok(())
}

pub fn validate_analysis_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    let len = trimmed.chars().count();
    if len == 0 || len > 120 || trimmed.chars().any(|character| character.is_control()) {
        return Err("Nom d'analyse invalide".into());
    }
    Ok(trimmed.to_string())
}

pub fn analyses_dir() -> PathBuf {
    crate::services::paths::data_dir().join("forecast-analyses")
}

pub fn index_path() -> PathBuf {
    analyses_dir().join("index.json")
}

pub async fn analysis_path_for_read(id: &str) -> std::io::Result<PathBuf> {
    crate::services::paths::data_file_for_read("forecast-analyses", &format!("{id}.json")).await
}

pub async fn analysis_path_for_write(id: &str) -> Result<PathBuf, String> {
    crate::services::paths::data_file_for_write("forecast-analyses", &format!("{id}.json"))
        .await
        .map_err(|_| "Erreur de sauvegarde".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_ids_and_names_are_bounded() {
        assert!(validate_analysis_id("550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(validate_analysis_id("../analysis").is_err());
        assert!(validate_analysis_name("Analyse").is_ok());
        assert!(validate_analysis_name("\n").is_err());
    }
}
