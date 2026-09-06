use super::{
    allowed_tool_count_for_test, clear_all_extensions_in, clear_extension, clear_session,
    is_extension_allowed, mark_allowed, mark_extension_allowed, AllowedTool,
};
use std::collections::HashMap;
use std::time::Instant;

#[tokio::test]
async fn bounds_tools_remembered_for_one_session() {
    let session_id = uuid::Uuid::new_v4().to_string();

    for index in 0..32 {
        mark_allowed(&session_id, &format!("tool-{index}")).await;
    }

    assert!(allowed_tool_count_for_test(&session_id).await <= 16);
    clear_session(&session_id).await;
}

#[tokio::test]
async fn never_remembers_tools_excluded_from_session_allow() {
    let session_id = uuid::Uuid::new_v4().to_string();

    mark_allowed(&session_id, "bash").await;
    mark_allowed(&session_id, "search_mcp_tools").await;

    assert_eq!(allowed_tool_count_for_test(&session_id).await, 0);
    clear_session(&session_id).await;
}

#[tokio::test]
async fn extension_authorization_is_bound_to_plugin_and_tool() {
    let session_id = uuid::Uuid::new_v4().to_string();
    // clear_extension operates across sessions; parallel tests need distinct plugins.
    let plugin = uuid::Uuid::new_v4().to_string();
    mark_extension_allowed(&session_id, &plugin, "shared-tool").await;

    assert!(is_extension_allowed(&session_id, &plugin, "shared-tool").await);
    assert!(!is_extension_allowed(&session_id, "plugin-b", "shared-tool").await);
    assert!(!is_extension_allowed(&session_id, &plugin, "other-tool").await);
    clear_session(&session_id).await;
}

#[tokio::test]
async fn clearing_one_extension_preserves_other_authorizations() {
    let session_id = uuid::Uuid::new_v4().to_string();
    mark_extension_allowed(&session_id, "plugin-a", "tool-a").await;
    mark_extension_allowed(&session_id, "plugin-b", "tool-b").await;

    clear_extension("plugin-a").await;

    assert!(!is_extension_allowed(&session_id, "plugin-a", "tool-a").await);
    assert!(is_extension_allowed(&session_id, "plugin-b", "tool-b").await);
    clear_session(&session_id).await;
}

#[test]
fn clearing_all_extensions_preserves_native_authorizations() {
    let native = AllowedTool {
        extension_id: String::new(),
        tool_name: "write_file".to_string(),
    };
    let extension = AllowedTool {
        extension_id: "plugin-a".to_string(),
        tool_name: "tool-a".to_string(),
    };
    let mut allowed = HashMap::from([(
        "session".to_string(),
        HashMap::from([
            (native.clone(), Instant::now()),
            (extension, Instant::now()),
        ]),
    )]);

    clear_all_extensions_in(&mut allowed);

    let tools = allowed.get("session").expect("native permission preserved");
    assert_eq!(tools.len(), 1);
    assert!(tools.contains_key(&native));
}
