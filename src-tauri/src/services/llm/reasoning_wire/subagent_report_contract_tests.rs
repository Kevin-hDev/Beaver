use super::{chat_text, replay, responses};
use crate::services::agent_local::ollama_wire;
use crate::services::agent_local::subagent_hidden_reports::{build_report, report_to_message};
use crate::services::agent_local::types_ollama::{ChatRequest, OllamaThink};
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, ContinuationUse, CredentialScope, ReplayTarget, RouteId,
};
use crate::services::reasoning_continuity::registry::{
    active_routes, ActivationState, AdapterId, ReplayRequirement,
};
use serde_json::json;

#[test]
fn every_live_user_continuation_accepts_the_shared_subagent_report() {
    let report = report_to_message(build_report(
        "child".into(),
        "Geminitor".into(),
        "explorer".into(),
        "completed".into(),
        "Rapport vérifié".into(),
    ));
    let mut checked = 0usize;

    for route in active_routes() {
        for model in route.models.iter().filter(|model| {
            model.activation == ActivationState::LiveValidated
                && model.continuation_use == ContinuationUse::UserContinuation
                && model.requirement != ReplayRequirement::Forbidden
        }) {
            let replay_target = ReplayTarget {
                route_id: route.route_id,
                model_id: model.model_id.into(),
                credential_scope: if route.route_id == RouteId::Ollama {
                    CredentialScope::local_uncredentialed()
                } else {
                    CredentialScope::authenticated("subagent-report-fixture").unwrap()
                },
                reasoning_mode: model.reasoning_mode,
                continuation_use: ContinuationUse::UserContinuation,
            };
            let target = ContinuationTarget::Replay(replay_target.clone());
            let messages = std::slice::from_ref(&report);

            let accepted = match route.adapter {
                AdapterId::ResponsesLocal => {
                    match responses::target_for_request(messages, Some(&target)) {
                        Ok(Some(_)) => Ok(()),
                        Ok(None) => Err(replay::ReplayApplyError::PayloadMismatch),
                        Err(error) => Err(error),
                    }
                }
                AdapterId::AnthropicBlocks => {
                    let mut payload = [json!({
                        "role": report.role.as_str(),
                        "content": [{"type": "text", "text": report.content.as_str()}],
                    })];
                    replay::apply_anthropic_messages(messages, Some(&target), &mut payload)
                        .and_then(|_| {
                            (payload[0]["role"] == "user")
                                .then_some(())
                                .ok_or(replay::ReplayApplyError::PayloadMismatch)
                        })
                }
                AdapterId::OllamaNative => {
                    let request = ChatRequest {
                        model: model.model_id.into(),
                        messages: Vec::new(),
                        stream: true,
                        tools: None,
                        options: None,
                        keep_alive: None,
                        think: Some(OllamaThink::Level(model.reasoning_mode.as_name().into())),
                        capture_reasoning: false,
                        live_replay_target: Some(replay_target),
                        #[cfg(debug_assertions)]
                        fixture_candidate: None,
                    };
                    ollama_wire::chat_request(&request, messages).and_then(|payload| {
                        (payload["messages"][0]["role"] == "user")
                            .then_some(())
                            .ok_or(replay::ReplayApplyError::PayloadMismatch)
                    })
                }
                AdapterId::GeminiParts
                | AdapterId::MistralChunks
                | AdapterId::CerebrasReasoning
                | AdapterId::OpenRouterDetails
                | AdapterId::ChatReasoning => {
                    let mut payload = json!({
                        "messages": [{
                            "role": report.role.as_str(),
                            "content": report.content.as_str(),
                        }],
                    });
                    chat_text::apply_continuity(messages, Some(&target), &mut payload).and_then(
                        |_| {
                            (payload["messages"][0]["role"] == "user")
                                .then_some(())
                                .ok_or(replay::ReplayApplyError::PayloadMismatch)
                        },
                    )
                }
            };

            assert!(
                accepted.is_ok(),
                "route={:?} model={} mode={:?} error={accepted:?}",
                route.route_id,
                model.model_id,
                model.reasoning_mode,
            );
            checked += 1;
        }
    }

    assert!(
        checked > 0,
        "the active registry must exercise at least one policy"
    );
}
