use super::super::profile_resolve::resolve_from_document;
use super::super::profile_store_document::CompressionProfileDocument;
use super::super::profile_types::CompressionTrigger;
use super::super::session_capabilities::SessionCompressionCapabilities;
use super::super::snapshot::CompressionSnapshot;
use super::super::summary_contract::ValidatedSummary;

pub(super) fn summary() -> ValidatedSummary {
    let content = super::super::summary_contract::required_sections()
        .into_iter()
        .map(|section| format!("{section}\nVerified continuation detail."))
        .collect::<Vec<_>>()
        .join("\n\n");
    ValidatedSummary {
        estimated_tokens: crate::services::token_counting::estimate_text_tokens(&content) as u32,
        content,
    }
}

pub(super) fn capabilities(chatbot: bool) -> SessionCompressionCapabilities {
    SessionCompressionCapabilities::from_runtime(
        chatbot,
        &[
            "web_search".into(),
            "web_fetch".into(),
            "read_file".into(),
            "delegate_task".into(),
            "todo_write".into(),
        ],
        true,
        true,
        true,
    )
    .expect("bounded capabilities")
}

pub(super) fn snapshot(
    session: &crate::services::agent_local::types_session::AgentSession,
    document: &CompressionProfileDocument,
    window: u64,
) -> CompressionSnapshot {
    CompressionSnapshot::capture(
        session,
        resolve_from_document(session.compression_profile_selection.as_ref(), document)
            .expect("profile"),
        window,
        capabilities(false),
        CompressionTrigger::Explicit,
    )
    .expect("snapshot")
    .with_runtime_context(Vec::new(), Vec::new(), 100_000)
    .expect("runtime context")
}
