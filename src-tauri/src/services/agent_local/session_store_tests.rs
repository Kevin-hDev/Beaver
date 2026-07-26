#[cfg(test)]
mod tests {
    use crate::services::agent_local::session_store::validate_session_id;

    #[test]
    fn valid_uuid_passes() {
        assert!(validate_session_id("550e8400-e29b-41d4-a716-446655440000").is_ok());
    }

    #[test]
    fn lowercase_hex_passes() {
        assert!(validate_session_id("abcdef01-2345-6789-abcd-ef0123456789").is_ok());
    }

    #[test]
    fn empty_id_blocked() {
        assert!(validate_session_id("").is_err());
    }

    #[test]
    fn path_traversal_blocked() {
        assert!(validate_session_id("../etc/passwd").is_err());
    }

    #[test]
    fn uppercase_blocked() {
        assert!(validate_session_id("ABCDEF01-2345-6789-ABCD-EF0123456789").is_err());
    }

    #[test]
    fn too_long_blocked() {
        let long = "a".repeat(65);
        assert!(validate_session_id(&long).is_err());
    }

    #[test]
    fn slash_in_id_blocked() {
        assert!(validate_session_id("abc/def").is_err());
    }

    #[test]
    fn null_byte_blocked() {
        assert!(validate_session_id("abc\0def").is_err());
    }
}
