use super::live_target_for_request;
use crate::services::reasoning_continuity::contract::{
    ContinuationUse, CredentialScope, ReasoningModeId, ReplayTarget, RouteId,
};

fn target(model_id: &str, reasoning_mode: ReasoningModeId) -> ReplayTarget {
    ReplayTarget {
        route_id: RouteId::Ollama,
        model_id: model_id.into(),
        credential_scope: CredentialScope::local_uncredentialed(),
        reasoning_mode,
        continuation_use: ContinuationUse::UserContinuation,
    }
}

#[test]
fn production_request_selects_exact_user_or_tool_target_for_each_validated_model() {
    for model in ["qwen3.5:4b", "gemma4:e2b-it-q4_K_M"] {
        let target = target(model, ReasoningModeId::Auto);
        assert_eq!(
            live_target_for_request(&target, false)
                .unwrap()
                .continuation_use,
            ContinuationUse::UserContinuation
        );
        assert_eq!(
            live_target_for_request(&target, true)
                .unwrap()
                .continuation_use,
            ContinuationUse::ToolContinuation
        );
    }
}

#[test]
fn production_target_blocks_neighbor_and_off_mode() {
    assert!(
        live_target_for_request(&target("deepseek-r1:latest", ReasoningModeId::Auto), true,)
            .is_err()
    );
    assert!(live_target_for_request(&target("qwen3.5:4b", ReasoningModeId::Off), true).is_err());
    assert!(
        live_target_for_request(&target("gemma4:e2b-it-q4_K_M", ReasoningModeId::Off), false,)
            .is_err()
    );
}
