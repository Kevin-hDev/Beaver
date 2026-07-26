use regex::Regex;
use std::sync::LazyLock;

static SESSION_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-f0-9\-]+$").unwrap());

pub fn validate_session_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 {
        return Err("Identifiant de session invalide".into());
    }
    if !SESSION_ID_REGEX.is_match(id) {
        return Err("Identifiant de session invalide".into());
    }
    Ok(())
}
