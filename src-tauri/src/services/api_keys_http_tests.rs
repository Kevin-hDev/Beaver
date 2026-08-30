use super::llm_probe;
use crate::services::llm::api_key_probe::{ProbeAuth, ProbeMethod};

#[test]
fn anthropic_uses_the_native_llm_probe_path() {
    let probe = llm_probe("anthropic")
        .expect("configurable LLM route")
        .expect("valid probe");
    assert_eq!(probe.method, ProbeMethod::Get);
    assert_eq!(probe.auth, ProbeAuth::XApiKey);
    assert_eq!(probe.headers, &[("anthropic-version", "2023-06-01")]);
}

#[test]
fn qwen_subscription_keys_are_rejected_before_network_use() {
    assert!(super::reject_unsupported_qwen_key("sk-sp-fixture").is_err());
}

#[test]
fn qwen_only_falls_back_to_chat_when_models_is_unsupported() {
    use super::QwenProbeAction;

    assert_eq!(super::qwen_probe_action(200), QwenProbeAction::Accept);
    assert_eq!(super::qwen_probe_action(404), QwenProbeAction::ChatFallback);
    assert_eq!(super::qwen_probe_action(405), QwenProbeAction::ChatFallback);
    assert_eq!(super::qwen_probe_action(401), QwenProbeAction::Reject);
    assert_eq!(super::qwen_probe_action(429), QwenProbeAction::Reject);
}

#[test]
fn a_stored_qwen_key_uses_the_connection_aware_probe() {
    assert!(super::uses_qwen_probe("qwen"));
    assert!(!super::uses_qwen_probe("anthropic"));
}
