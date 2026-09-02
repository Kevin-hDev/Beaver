use super::types::ExtensionTool;
use super::view::ExtensionView;
use serde_json::json;

#[test]
fn ipc_view_includes_runtime_contributions_while_storage_record_omits_them() {
    let mut record = super::builtin::records()
        .expect("builtin catalog should load")
        .remove(0);
    record.contributions.tools.push(ExtensionTool {
        name: "beaver.test.tool".to_string(),
        description: "Runtime contribution".to_string(),
        parameters: json!({ "type": "object" }),
        effect: "unknown".to_string(),
        replaces_core: false,
    });

    let stored = serde_json::to_value(&record).expect("record should serialize");
    let ipc =
        serde_json::to_value(ExtensionView::from(record)).expect("extension view should serialize");

    assert!(stored.get("contributions").is_none());
    assert_eq!(
        ipc.pointer("/contributions/tools/0/name"),
        Some(&json!("beaver.test.tool")),
    );
    assert_eq!(ipc.pointer("/contributions/events"), Some(&json!([])));
}
