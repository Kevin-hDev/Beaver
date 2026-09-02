use super::*;
use serde_json::json;

#[test]
fn chat_policy_has_exactly_two_native_tools() {
    assert!(is_chat_tool("web_search"));
    assert!(is_chat_tool("web_fetch"));
    assert!(!is_chat_tool("bash"));
    assert!(!is_chat_tool("search_extension_tools"));
}

#[test]
fn inactive_replacements_fall_back_to_core_but_other_plugins_fail_closed() {
    assert_eq!(dynamic_route(true, false, true), Ok(false));
    assert_eq!(dynamic_route(true, true, true), Ok(true));
    assert_eq!(
        dynamic_route(true, false, false),
        Err("Extension indisponible.")
    );
}

#[tokio::test]
async fn chat_rejects_an_agentic_call_before_dispatch() {
    let result = dispatch_for_mode(
        "bash",
        &json!({"command": "pwd"}),
        std::path::Path::new("."),
        "test-session",
        CancellationToken::new(),
        true,
    )
    .await;

    assert!(result.is_error);
    assert_eq!(result.content, "Outil indisponible dans ce mode.");
}

#[tokio::test]
async fn chat_rejects_an_extension_call_before_dispatch() {
    let result = dispatch_for_mode(
        "beaver.office.documents.create",
        &json!({}),
        std::path::Path::new("."),
        "test-session",
        CancellationToken::new(),
        true,
    )
    .await;

    assert!(result.is_error);
    assert_eq!(
        result.error.as_ref().map(|error| error.code.as_ref()),
        Some("tool_unavailable_in_mode"),
    );
}
