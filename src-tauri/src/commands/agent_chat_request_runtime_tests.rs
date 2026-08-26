use crate::models::agent_turn_contract::ChatStreamRequestInput;
use serde_json::{json, Value};
use tauri::ipc::InvokeBody;
use tauri::webview::{InvokeRequest, WebviewWindowBuilder};

#[tauri::command]
fn deserialize_chat_stream_request(_request: ChatStreamRequestInput) -> bool {
    true
}

#[test]
fn tauri_rejects_legacy_history_and_frontend_runtime_controls() {
    let app = tauri::test::mock_builder()
        .invoke_handler(tauri::generate_handler![deserialize_chat_stream_request])
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("build isolated Tauri app");
    let window = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("build isolated webview");

    for body in malformed_requests() {
        let response = tauri::test::get_ipc_response(
            &window,
            InvokeRequest {
                cmd: "deserialize_chat_stream_request".into(),
                callback: tauri::ipc::CallbackFn(1),
                error: tauri::ipc::CallbackFn(2),
                url: "http://tauri.localhost".parse().unwrap(),
                body: InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        );
        assert!(response.is_err());
    }
}

#[test]
fn production_root_exposes_only_the_strict_request_and_injected_state() {
    let source = include_str!("agent_chat.rs");
    assert!(source.contains("request: ChatStreamRequestInput"));
    for forbidden in [
        "messages:",
        "think:",
        "supports_thinking:",
        "reasoning_mode:",
    ] {
        assert!(!source
            .split("pub async fn chat_stream")
            .nth(1)
            .unwrap()
            .split(") -> Result")
            .next()
            .unwrap()
            .contains(forbidden));
    }
}

fn malformed_requests() -> Vec<Value> {
    vec![
        json!({"sessionId": "session", "messages": []}),
        request_with(json!({"messages": []})),
        request_with(json!({"think": true})),
        request_with(json!({"supportsThinking": true})),
        request_with(json!({"reasoningMode": "high"})),
        request_with(json!({"unknown": true})),
    ]
}

fn request_with(extra: Value) -> Value {
    let mut request = json!({
        "sessionId": "session",
        "model": "model",
        "provider": "ollama",
        "turn": {"type": "new", "input": {"content": "hi", "files": [], "skills": []}},
        "workingDir": null,
        "permissionMode": null,
        "planMode": null
    });
    request
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    json!({"request": request})
}
