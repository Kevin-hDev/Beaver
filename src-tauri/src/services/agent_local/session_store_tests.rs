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
        let path =
            crate::services::paths::data_file_for_write("agent-sessions", &format!("{id}.json"))
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

    #[tokio::test]
    async fn persists_the_latest_context_snapshot() {
        let session =
            super::super::create_full("context snapshot", "model", "provider", false, None)
                .await
                .expect("create session");
        let message_id = uuid::Uuid::new_v4().to_string();
        let message = crate::services::agent_local::types_session::AgentMessage {
            id: message_id.clone(),
            turn_id: crate::services::agent_local::types_session::AgentMessage::new_turn_id(),
            role: "assistant".into(),
            content: "answer".into(),
            thinking: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            continuation: None,
            replay_source: None,
            tool_activities: None,
            segments: None,
            files: vec![],
            timestamp: chrono::Utc::now(),
            tokens: 0,
            work_duration_ms: None,
            skill_names: None,
            skill_ids: None,
            stream_run_id: None,
            stream_part: None,
        };

        super::super::add_messages_with_context(
            &session.id,
            vec![message],
            7,
            Some(4_321),
            Some(4_000),
        )
        .await
        .expect("save context snapshot");
        let saved = super::super::get(&session.id)
            .await
            .expect("reload session");

        assert_eq!(saved.accumulated_tokens, 2);
        assert_eq!(saved.context_tokens, Some(4_000));

        super::super::add_messages_with_context(&session.id, vec![], 0, Some(3_000), None)
            .await
            .expect("ignore snapshot without limit");
        let unbounded = super::super::get(&session.id)
            .await
            .expect("reload unbounded");
        assert_eq!(unbounded.context_tokens, None);

        crate::services::agent_local::session_ops::truncate_and_replace(
            &session.id,
            &message_id,
            None,
        )
        .await
        .expect("edit session history");
        let edited = super::super::get(&session.id).await.expect("reload edit");
        assert_eq!(edited.accumulated_tokens, 2);
        assert_eq!(edited.context_tokens, None);
        super::super::delete_one(&session.id)
            .await
            .expect("delete session");
    }

    #[tokio::test]
    async fn delete_removes_backup_and_known_atomic_temps() {
        let session =
            super::super::create_full("artifact cleanup", "model", "provider", false, None)
                .await
                .expect("create session");
        let directory = crate::services::paths::data_dir().join("agent-sessions");
        let main = directory.join(format!("{}.json", session.id));
        let backup = directory.join(format!("{}.json.v1.bak", session.id));
        let temp = directory.join(format!(
            ".{}.json.0123456789abcdef0123456789abcdef.tmp",
            session.id
        ));
        crate::services::private_store::atomic_write(&backup, b"fixture backup").unwrap();
        crate::services::private_store::atomic_write(&temp, b"fixture temp").unwrap();

        super::super::delete_one(&session.id)
            .await
            .expect("delete with artifacts");

        assert!(!main.exists());
        assert!(!backup.exists());
        assert!(!temp.exists());
    }

    #[tokio::test]
    async fn legacy_frontend_cannot_erase_or_inject_private_continuity_state() {
        use crate::services::reasoning_continuity::contract::{
            ContractId, CredentialScope, ReasoningModeId, RouteId,
        };
        use crate::services::reasoning_continuity::envelope::{
            CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
        };

        let mut session =
            super::super::create_full("continuation guard", "fixture-model", "ollama", false, None)
                .await
                .expect("create session");
        let envelope = ReasoningEnvelope::new(
            ContractId::OllamaNativeV1,
            ReasoningSource {
                route_id: RouteId::Ollama,
                model_id: "fixture-model".into(),
                credential_scope: CredentialScope::local_uncredentialed(),
                reasoning_mode: ReasoningModeId::Auto,
            },
            CompletionState::Complete,
            ContinuationState::OllamaNative {
                thinking: "opaque fixture".into(),
            },
            Vec::new(),
        );
        session
            .messages
            .push(crate::services::agent_local::types_session::AgentMessage {
                id: uuid::Uuid::new_v4().to_string(),
                turn_id: "turn-preserved".into(),
                role: "assistant".into(),
                content: "visible".into(),
                thinking: None,
                tool_calls: None,
                tool_name: None,
                tool_call_id: None,
                continuation: Some(envelope.clone()),
                replay_source: None,
                tool_activities: None,
                segments: None,
                files: vec![],
                timestamp: chrono::Utc::now(),
                tokens: 0,
                work_duration_ms: None,
                skill_names: None,
                skill_ids: None,
                stream_run_id: None,
                stream_part: None,
            });
        super::super::save(&session)
            .await
            .expect("seed continuation");
        let appended = crate::services::agent_local::types_session::AgentMessage {
            id: uuid::Uuid::new_v4().to_string(),
            turn_id: "turn-appended".into(),
            role: "user".into(),
            content: "next".into(),
            thinking: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            continuation: None,
            replay_source: None,
            tool_activities: None,
            segments: None,
            files: vec![],
            timestamp: chrono::Utc::now(),
            tokens: 0,
            work_duration_ms: None,
            skill_names: None,
            skill_ids: None,
            stream_run_id: None,
            stream_part: None,
        };
        super::super::add_messages(&session.id, vec![appended], 0)
            .await
            .expect("append visible message");
        let restored = super::super::get(&session.id).await.expect("reload");
        assert_eq!(restored.messages[0].continuation, Some(envelope));
        let mut forged = restored.messages[1].clone();
        forged.id = uuid::Uuid::new_v4().to_string();
        forged.turn_id = crate::services::agent_local::types_session::AgentMessage::new_turn_id();
        forged.replay_source = Some(ReasoningSource {
            route_id: RouteId::Ollama,
            model_id: "fixture-model".into(),
            credential_scope: CredentialScope::local_uncredentialed(),
            reasoning_mode: ReasoningModeId::Auto,
        });
        assert!(super::super::add_messages(&session.id, vec![forged], 0)
            .await
            .is_err());
        assert_eq!(
            super::super::get(&session.id)
                .await
                .expect("reload after rejected injection")
                .messages
                .len(),
            2
        );
        super::super::delete_one(&session.id)
            .await
            .expect("cleanup");
    }
}
