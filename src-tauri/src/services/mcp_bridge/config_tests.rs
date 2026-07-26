use super::config::{self, StoredConnector, MAX_CONNECTORS};
use super::trusted::is_trusted_endpoint_for_connector;

fn connector(id: &str) -> StoredConnector {
    StoredConnector {
        id: id.to_string(),
        status: "connected".to_string(),
        enabled_in_chat: true,
        endpoint: Some("https://mcp.notion.com/mcp".to_string()),
        install_command: None,
        env_keys: None,
    }
}

#[test]
fn validates_catalog_endpoint_for_matching_connector() {
    assert!(is_trusted_endpoint_for_connector(
        "notion",
        "https://mcp.notion.com/mcp"
    ));
}

#[test]
fn rejects_catalog_endpoint_for_wrong_connector() {
    let mut c = connector("sentry");
    c.endpoint = Some("https://mcp.notion.com/mcp".to_string());
    assert!(config::validate_connector(&c).is_err());
}

#[test]
fn rejects_invalid_status() {
    let mut c = connector("notion");
    c.status = "pending".to_string();
    assert!(config::validate_connector(&c).is_err());
}

#[test]
fn max_connector_limit_is_bounded() {
    assert_eq!(MAX_CONNECTORS, 32);
}

#[test]
fn imessage_install_command_is_forced() {
    let c = StoredConnector {
        id: "imessage".to_string(),
        status: "connected".to_string(),
        enabled_in_chat: true,
        endpoint: None,
        install_command: Some("npx bad".to_string()),
        env_keys: None,
    };
    let cmd = config::install_command_for(&c).unwrap();
    assert!(cmd.contains("jsr:@wyattjoh/imessage-mcp@0.4.2"));
    assert!(!cmd.contains("npx bad"));
}

#[test]
fn rejects_forbidden_env_key() {
    let mut c = StoredConnector {
        id: "huggingface".to_string(),
        status: "connected".to_string(),
        enabled_in_chat: true,
        endpoint: None,
        install_command: Some("npx @llmindset/hf-mcp-server@0.3.13".to_string()),
        env_keys: Some(vec!["NODE_OPTIONS".to_string()]),
    };
    assert!(config::validate_connector(&c).is_err());

    c.env_keys = Some(vec!["HF_TOKEN".to_string()]);
    assert!(config::validate_connector(&c).is_ok());
}

#[test]
fn load_migrates_legacy_stdio_commands() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mcp-connectors.json");
    let legacy = vec![
        StoredConnector {
            id: "context7".to_string(),
            status: "connected".to_string(),
            enabled_in_chat: true,
            endpoint: None,
            install_command: Some("npx @upstash/context7-mcp@2.2.3".to_string()),
            env_keys: None,
        },
        StoredConnector {
            id: "huggingface".to_string(),
            status: "connected".to_string(),
            enabled_in_chat: true,
            endpoint: None,
            install_command: Some("npx @llmindset/hf-mcp-server@0.3.11".to_string()),
            env_keys: Some(vec!["HF_TOKEN".to_string()]),
        },
    ];
    std::fs::write(&path, serde_json::to_vec_pretty(&legacy).unwrap()).unwrap();

    let loaded = config::load_from_path(&path).unwrap();

    assert_eq!(
        loaded[0].install_command.as_deref(),
        Some("npx @upstash/context7-mcp@2.2.5")
    );
    assert_eq!(
        loaded[1].install_command.as_deref(),
        Some("npx @llmindset/hf-mcp-server@0.3.13")
    );
    let saved = std::fs::read_to_string(path).unwrap();
    assert!(!saved.contains("2.2.3"));
    assert!(!saved.contains("0.3.11"));
}
