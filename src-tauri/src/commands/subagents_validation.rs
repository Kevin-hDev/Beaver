pub(crate) fn validate_session_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 || id.contains("..") || id.contains('/') {
        return Err("Identifiant de session invalide".to_string());
    }
    Ok(())
}

#[cfg(test)]
#[allow(dead_code)]
pub fn validate_session_id_for_test(id: &str) -> Result<(), String> {
    validate_session_id(id)
}
