pub(super) fn message(kind: std::io::ErrorKind) -> String {
    match kind {
        std::io::ErrorKind::NotFound => "Shell utilisateur indisponible.".to_string(),
        std::io::ErrorKind::PermissionDenied => "Lancement du shell refusé.".to_string(),
        _ => "Lancement du shell impossible.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::message;

    #[test]
    fn preserves_actionable_spawn_error_classes() {
        let denied = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        let missing = std::io::Error::from(std::io::ErrorKind::NotFound);

        assert!(message(denied.kind()).contains("refusé"));
        assert!(message(missing.kind()).contains("indisponible"));
    }
}
