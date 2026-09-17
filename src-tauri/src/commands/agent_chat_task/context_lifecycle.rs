use super::{api, chat_engine, common, compress, ollama, ChatEngine};
use crate::services::agent_local::agent_loop_finish::CompletedStreamTurn;
use crate::services::agent_local::context_usage_record::ContextPreparationState;
use crate::services::agent_local::conversation_journal::ConversationJournal;
use crate::services::agent_local::types_ollama::ChatMessage;

pub(super) async fn activate(journal: Option<&ConversationJournal>) -> Result<(), String> {
    if let Some(journal) = journal {
        journal.activate_context_request().await?;
    }
    Ok(())
}

pub(super) async fn finish(
    journal: Option<&ConversationJournal>,
    outcome: &Result<CompletedStreamTurn, String>,
) -> Result<(), String> {
    let Some(journal) = journal else {
        return Ok(());
    };
    journal
        .finish_context_request(terminal_state(outcome))
        .await
}

pub(super) fn terminal_state(
    outcome: &Result<CompletedStreamTurn, String>,
) -> ContextPreparationState {
    match outcome {
        Ok(_) => ContextPreparationState::Completed,
        Err(error) if error == "Annulé" => ContextPreparationState::Interrupted,
        Err(_) => ContextPreparationState::Failed,
    }
}

pub(super) async fn run(
    params: super::StreamTaskParams,
    messages: Vec<ChatMessage>,
    journal: &mut Option<ConversationJournal>,
    mode: common::StreamMode,
) -> Result<CompletedStreamTurn, String> {
    if compress::is_compress_command(&messages) {
        let working_dir = common::resolve_working_dir(&params.working_dir)?;
        common::update_working_dir(&params.session_id, &working_dir).await?;
        compress::handle_compress_command(
            &params.on_event,
            &params.session_id,
            &params.request_id,
            &messages,
            &params.model,
            &params.provider,
            &params.tools,
            mode.is_chat,
            params.plan_mode.unwrap_or(false),
            &working_dir,
            params.cancel.clone(),
        )
        .await?;
        return Ok(CompletedStreamTurn::compression(messages));
    }
    let response_language = response_language(&params);
    let journal = journal
        .as_mut()
        .ok_or_else(|| "conversation_admission_failed".to_string())?;
    if chat_engine(&params.provider) == ChatEngine::Ollama {
        ollama::run(params, messages, mode, response_language, journal).await
    } else {
        api::run(params, messages, mode, response_language, journal).await
    }
}

fn response_language(params: &super::StreamTaskParams) -> String {
    #[cfg(debug_assertions)]
    if params.fixture_run.is_some() {
        return String::new();
    }
    #[cfg(not(debug_assertions))]
    let _ = params;
    common::response_language()
}
