use super::{Manifest, MANIFEST_ENV};

pub(super) fn run() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(recover());
}

async fn recover() {
    let manifest: Manifest = serde_json::from_slice(
        &std::fs::read(std::env::var_os(MANIFEST_ENV).expect("manifest path"))
            .expect("read manifest"),
    )
    .expect("manifest");
    copy_session_state(&manifest);
    super::super::stream_recovery_startup::recover_all().await;
    let recovered = super::super::session_store::get(&manifest.session_id)
        .await
        .expect("recovered session");
    assert!(recovered.messages.iter().any(|message| {
        message.content == "visible work" && message.thinking.as_deref() == Some("visible thinking")
    }));
    assert!(recovered.messages.iter().any(|message| {
        message.role == "tool" && message.content.contains("completed read proof")
    }));
    let ids = recovered
        .messages
        .iter()
        .map(|message| message.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(ids.len(), recovered.messages.len());
    assert!(recovered.messages.iter().any(|message| {
        message.role == "tool"
            && message.tool_call_id.as_deref() == Some("call-second")
            && message.content.contains("tool_interrupted")
    }));
    super::super::conversation_admission::new_turn(
        &manifest.session_id,
        crate::services::agent_local::conversation_input::ResolvedTurnInput {
            user_content: "continue after recovery".into(),
            provider_content: "continue after recovery".into(),
            files: Vec::new(),
            images: Vec::new(),
            skills: Vec::new(),
        },
        target(),
    )
    .await
    .expect("new turn after recovery");
    super::super::session_store::delete_one(&manifest.session_id)
        .await
        .expect("cleanup recovery child");
}

fn target() -> crate::services::reasoning_continuity::contract::ReplayTarget {
    use crate::services::reasoning_continuity::contract::*;
    ReplayTarget {
        route_id: RouteId::Ollama,
        model_id: "qwen3.5:4b".into(),
        credential_scope: CredentialScope::local_uncredentialed(),
        reasoning_mode: ReasoningModeId::Off,
        continuation_use: ContinuationUse::UserContinuation,
    }
}

fn copy_session_state(manifest: &Manifest) {
    let destination = crate::services::paths::data_dir();
    let sessions = destination.join("agent-sessions");
    let recovery = destination
        .join("agent-stream-recovery")
        .join(&manifest.session_id);
    std::fs::create_dir_all(&sessions).expect("session destination");
    std::fs::create_dir_all(&recovery).expect("recovery destination");
    std::fs::copy(
        manifest
            .data_dir
            .join("agent-sessions")
            .join(format!("{}.json", manifest.session_id)),
        sessions.join(format!("{}.json", manifest.session_id)),
    )
    .expect("copy session");
    for entry in std::fs::read_dir(
        manifest
            .data_dir
            .join("agent-stream-recovery")
            .join(&manifest.session_id),
    )
    .expect("source recovery")
    {
        let entry = entry.expect("recovery entry");
        std::fs::copy(entry.path(), recovery.join(entry.file_name())).expect("copy recovery log");
    }
}
