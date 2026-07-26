#[cfg(test)]
mod tests {
    use crate::commands::subagents::validate_session_id_for_test;

    #[test]
    fn test_valid_uuid() {
        let id = uuid::Uuid::new_v4().to_string();
        assert!(
            validate_session_id_for_test(&id).is_ok(),
            "Un UUID valide doit passer la validation"
        );
    }

    #[test]
    fn test_empty_id_rejected() {
        assert!(
            validate_session_id_for_test("").is_err(),
            "Une chaîne vide doit être rejetée"
        );
    }

    #[test]
    fn test_too_long_id_rejected() {
        let long_id = "a".repeat(65);
        assert!(
            validate_session_id_for_test(&long_id).is_err(),
            "Une chaîne de plus de 64 chars doit être rejetée"
        );
    }

    #[test]
    fn test_path_traversal_rejected() {
        assert!(
            validate_session_id_for_test("../../etc/passwd").is_err(),
            "Un ID contenant '..' doit être rejeté"
        );
    }

    #[test]
    fn test_slash_rejected() {
        assert!(
            validate_session_id_for_test("foo/bar").is_err(),
            "Un ID contenant '/' doit être rejeté"
        );
    }
}
