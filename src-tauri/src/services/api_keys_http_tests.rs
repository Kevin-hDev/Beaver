use super::llm_probe;
use crate::services::llm::api_key_probe::{ProbeAuth, ProbeMethod};

#[test]
fn anthropic_candidate_uses_the_native_llm_probe_path() {
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
