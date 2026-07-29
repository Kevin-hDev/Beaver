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

    #[tokio::test]
    async fn invalid_session_file_can_still_be_deleted() {
        let id = uuid::Uuid::new_v4().to_string();
        let path = crate::services::paths::data_file_for_write(
            "agent-sessions",
            &format!("{id}.json"),
        )
        .await
        .expect("session path");
        tokio::fs::write(&path, b"{invalid")
            .await
            .expect("write invalid session");

        super::super::delete_one(&id)
            .await
            .expect("delete invalid session");

        assert!(!path.exists());
    }

    #[tokio::test]
    async fn deleting_a_session_preserves_its_managed_files() {
        let mut session =
            super::super::create_full("preserve files", "model", "provider", false, None)
                .await
                .expect("create session");
        let suffix = session.id.chars().take(8).collect::<String>();
        let root = crate::services::paths::data_dir()
            .join("session-workspaces")
            .join("2026-07-30")
            .join(format!("preserve-{suffix}"));
        let work = root.join("work");
        let outputs = root.join("outputs");
        tokio::fs::create_dir_all(&work).await.expect("create work");
        tokio::fs::create_dir_all(&outputs)
            .await
            .expect("create outputs");
        tokio::fs::write(work.join("draft.md"), b"draft")
            .await
            .expect("write draft");
        tokio::fs::write(outputs.join("report.md"), b"report")
            .await
            .expect("write report");
        session.working_dir = work.to_string_lossy().to_string();
        session.working_dir_managed = true;
        super::super::save(&session).await.expect("save session");

        super::super::delete_one(&session.id)
            .await
            .expect("delete session");

        assert!(work.join("draft.md").is_file());
        assert!(outputs.join("report.md").is_file());
        tokio::fs::remove_dir_all(root)
            .await
            .expect("remove test workspace");
    }
}
