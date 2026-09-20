use super::registry::{
    call_enabled_tool, mutate_identity_if_generation, select_exact_tool, EnabledConnector,
};
use super::transport::{McpCallError, McpToolCatalog, McpToolDef, McpToolResult, McpTransport};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn late_oauth_commit_cannot_revive_an_old_generation() {
    let wrote = AtomicUsize::new(0);
    let result = mutate_identity_if_generation("test-late-oauth", u64::MAX, |_| {
        wrote.fetch_add(1, Ordering::SeqCst);
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(wrote.load(Ordering::SeqCst), 0);
}

struct CountingTransport(Arc<AtomicUsize>);

#[async_trait::async_trait]
impl McpTransport for CountingTransport {
    async fn list_tools(&self) -> Result<McpToolCatalog, String> {
        Ok(McpToolCatalog {
            tools: Vec::new(),
            cache_ttl: None,
        })
    }

    async fn call_tool(
        &self,
        _name: &str,
        _args: serde_json::Value,
    ) -> Result<McpToolResult, McpCallError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(McpToolResult {
            content: "unexpected".to_string(),
            is_error: false,
        })
    }
}

fn tool(name: &str) -> McpToolDef {
    McpToolDef {
        name: name.to_string(),
        description: None,
        input_schema: Some(json!({"type": "object"})),
    }
}

#[test]
fn only_the_exact_last_listed_tool_is_selected() {
    let tools = vec![tool("read_public"), tool("read_private")];
    assert_eq!(
        select_exact_tool(&tools, "read_public").unwrap().name,
        "read_public"
    );
    assert!(select_exact_tool(&tools, "read").is_err());
    assert!(select_exact_tool(&tools, "hidden_tool").is_err());
}

#[test]
fn duplicate_tool_names_are_rejected_at_call_time_too() {
    let tools = vec![tool("duplicate"), tool("duplicate")];
    assert!(select_exact_tool(&tools, "duplicate").is_err());
}

#[tokio::test]
async fn stale_connector_is_refused_before_business_send() {
    let calls = Arc::new(AtomicUsize::new(0));
    let connector = EnabledConnector {
        id: "test-stale-connector".to_string(),
        transport: Arc::new(CountingTransport(calls.clone())),
        generation: u64::MAX,
    };
    let result = call_enabled_tool(&connector, "echo", json!({})).await;
    assert!(matches!(result, Err(McpCallError::Unavailable)));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}
