use super::api_key_probe::{resolve, ProbeAuth, ProbeMethod};

#[test]
fn api_key_probe_declares_active_routes_without_network_calls() {
    let openai = resolve("openai").unwrap();
    assert_eq!(openai.method, ProbeMethod::Get);
    assert_eq!(openai.url, "https://api.openai.com/v1/models");
    assert_eq!(openai.auth, ProbeAuth::Bearer);
    assert!(openai.body.is_none());

    let zai = resolve("zai").unwrap();
    assert_eq!(zai.method, ProbeMethod::Post);
    assert_eq!(zai.url, "https://api.z.ai/api/paas/v4/chat/completions");
    assert_eq!(zai.auth, ProbeAuth::Bearer);
    assert_eq!(zai.body.as_ref().unwrap()["max_tokens"], 1);

    for route in ["xai-oauth", "moonshot-oauth", "codex-oauth", "ollama"] {
        assert_eq!(resolve(route), Err("provider_configuration_invalid"));
    }
    assert_eq!(resolve("unknown"), Err("provider_configuration_invalid"));
}

#[test]
fn anthropic_probe_uses_models_and_version_header() {
    let probe = resolve("anthropic").expect("anthropic probe");
    assert_eq!(probe.method, ProbeMethod::Get);
    assert_eq!(probe.url, "https://api.anthropic.com/v1/models");
    assert_eq!(probe.auth, ProbeAuth::XApiKey);
    assert_eq!(probe.headers, &[("anthropic-version", "2023-06-01")]);
    assert!(probe.body.is_none());
}
