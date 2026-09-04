use super::super::tool_dispatch_trace::DispatchTrace;
use super::*;
use serde_json::json;

#[test]
fn chat_policy_has_exactly_two_native_tools() {
    assert!(is_chat_tool("web_search"));
    assert!(is_chat_tool("web_fetch"));
    assert!(!is_chat_tool("bash"));
    assert!(!is_chat_tool("list_extensions"));
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
        None,
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
        None,
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

#[tokio::test]
async fn inspection_without_an_exact_request_correlation_fails_closed() {
    let result = super::super::tool_dispatcher::dispatch_inner(
        crate::services::extensions::INSPECT_EXTENSIONS_TOOL_NAME,
        &json!({"ids": ["example.documents"]}),
        std::path::Path::new("."),
        DispatchTrace {
            session_id: "test-session",
            request_id: None,
        },
        CancellationToken::new(),
        None,
        None,
    )
    .await;

    assert_eq!(
        result.error.as_ref().map(|error| error.code.as_ref()),
        Some(crate::services::extensions::error_codes::INSPECTION_UNAVAILABLE)
    );
}
