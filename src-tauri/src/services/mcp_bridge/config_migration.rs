use super::config::StoredConnector;
use super::stdio_catalog;

pub fn normalize_list(connectors: &mut [StoredConnector]) -> bool {
    let mut changed = false;
    for connector in connectors {
        changed |= normalize_connector(connector);
    }
    changed
}

fn normalize_connector(connector: &mut StoredConnector) -> bool {
    let Some(canonical) = stdio_catalog::install_command(&connector.id) else {
        return false;
    };
    if connector.install_command.as_deref() == Some(canonical.as_str()) {
        return false;
    }
    connector.install_command = Some(canonical);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connector(id: &str, install_command: Option<&str>) -> StoredConnector {
        StoredConnector {
            id: id.to_string(),
            status: "connected".to_string(),
            enabled_in_chat: true,
            endpoint: None,
            install_command: install_command.map(str::to_string),
            env_keys: None,
        }
    }

    #[test]
    fn migrates_legacy_stdio_versions() {
        let mut list = vec![
            connector("context7", Some("npx @upstash/context7-mcp@2.2.3")),
            connector("reddit", Some("npx reddit-mcp-server@1.2.1")),
            connector("producthunt", Some("uvx product-hunt-mcp")),
            connector("huggingface", Some("npx @llmindset/hf-mcp-server@0.3.11")),
        ];

        assert!(normalize_list(&mut list));

        assert_eq!(
            list[0].install_command.as_deref(),
            Some("npx @upstash/context7-mcp@2.2.5")
        );
        assert_eq!(
            list[1].install_command.as_deref(),
            Some("npx reddit-mcp-server@1.4.5")
        );
        assert_eq!(
            list[2].install_command.as_deref(),
            Some("uvx product-hunt-mcp==0.1.0")
        );
        assert_eq!(
            list[3].install_command.as_deref(),
            Some("npx @llmindset/hf-mcp-server@0.3.13")
        );
    }

    #[test]
    fn migrates_legacy_imessage_permissions() {
        let mut list = vec![connector(
            "imessage",
            Some("deno run --allow-read --allow-write --allow-env --allow-sys --allow-ffi --allow-net jsr:@wyattjoh/imessage-mcp"),
        )];

        assert!(normalize_list(&mut list));
        assert_eq!(
            list[0].install_command.as_deref(),
            Some(stdio_catalog::IMESSAGE_INSTALL_COMMAND)
        );
    }
}
