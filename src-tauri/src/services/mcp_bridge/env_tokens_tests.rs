use super::config::StoredConnector;
use super::env_tokens::{validate, EnvTokenInput};

fn connector() -> StoredConnector {
    StoredConnector {
        id: "huggingface".to_string(),
        status: "connected".to_string(),
        enabled_in_chat: true,
        endpoint: None,
        install_command: Some("npx @llmindset/hf-mcp-server@0.3.13".to_string()),
        env_keys: Some(vec!["HF_TOKEN".to_string()]),
    }
}

fn token(key: &str, value: &str) -> EnvTokenInput {
    EnvTokenInput {
        env_key: key.to_string(),
        value: value.to_string(),
    }
}

#[test]
fn requires_exactly_the_declared_environment_secrets() {
    assert!(validate(&connector(), &[token("HF_TOKEN", "secret")]).is_ok());
    assert!(validate(&connector(), &[]).is_err());
    assert!(validate(&connector(), &[token("OTHER_TOKEN", "secret")]).is_err());
    assert!(validate(
        &connector(),
        &[token("HF_TOKEN", "one"), token("HF_TOKEN", "two")]
    )
    .is_err());
}

#[test]
fn invalid_secret_values_are_rejected() {
    assert!(validate(&connector(), &[token("HF_TOKEN", "")]).is_err());
    assert!(validate(&connector(), &[token("HF_TOKEN", &"x".repeat(8193))]).is_err());
}
