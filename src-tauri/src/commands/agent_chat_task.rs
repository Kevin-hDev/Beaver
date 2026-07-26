mod api;
mod api_images;
pub(crate) mod common;
mod compress;
mod gemma4_thinking_guard;
mod ollama;
mod params;

pub(crate) use params::{StreamCapabilityHints, StreamTaskParams};

use crate::services::agent_local::types_ollama::ChatMessage;

pub(crate) use common::merge_personality;

#[derive(Debug, PartialEq, Eq)]
enum ChatEngine {
    Ollama,
    NativeApi,
}

fn chat_engine(provider: &str) -> ChatEngine {
    if provider == "ollama" {
        ChatEngine::Ollama
    } else {
        ChatEngine::NativeApi
    }
}

pub(crate) async fn run_stream_task(
    mut params: StreamTaskParams,
) -> Result<Vec<ChatMessage>, String> {
    if let Some(permission_emitter) = params.permission_emitter.take() {
        params.on_event = params.on_event.with_permission_emitter(permission_emitter);
    }
    if compress::is_compress_command(&params.messages) {
        let working_dir = common::resolve_working_dir(&params.working_dir)?;
        common::update_working_dir(&params.session_id, &working_dir).await;
        compress::handle_compress_command(
            &params.on_event,
            &params.session_id,
            &params.request_id,
            &params.messages,
            &params.model,
            &params.provider,
            &working_dir,
            params.cancel.clone(),
        )
        .await?;
        return Ok(params.messages);
    }

    let mode = common::resolve_permission_mode(params.permission_mode_override.as_deref()).await;
    let response_language = common::response_language();

    if chat_engine(&params.provider) == ChatEngine::Ollama {
        ollama::run(params, mode, response_language).await
    } else {
        api::run(params, mode, response_language).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grok_and_kimi_oauth_use_the_native_agent_loop() {
        assert_eq!(chat_engine("xai-oauth"), ChatEngine::NativeApi);
        assert_eq!(chat_engine("moonshot-oauth"), ChatEngine::NativeApi);
        assert_eq!(chat_engine("xai"), ChatEngine::NativeApi);
        assert_eq!(chat_engine("moonshot"), ChatEngine::NativeApi);
    }
}
