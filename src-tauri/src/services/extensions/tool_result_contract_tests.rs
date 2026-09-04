use super::tool_result_contract::{validate, FilePurpose, ToolResultBlock, ToolResultContent};

#[test]
fn legacy_text_and_bounded_rich_blocks_validate() {
    assert!(validate(&ToolResultContent::Text("legacy".into())).is_ok());
    assert!(validate(&ToolResultContent::Blocks(vec![ToolResultBlock::Text { text: "text".into() }])).is_ok());
}

#[test]
fn rejects_more_than_the_generated_block_limit() {
    let blocks = (0..=super::types::MAX_RESULT_BLOCKS).map(|_| ToolResultBlock::Text { text: String::new() }).collect();
    assert!(validate(&ToolResultContent::Blocks(blocks)).is_err());
}

#[test]
fn generated_block_and_file_limits_accept_their_exact_boundaries() {
    let files = (0..super::types::MAX_RESULT_FILES).map(|index| ToolResultBlock::File {
        path: format!("result-{index}.txt"),
        purpose: FilePurpose::Artifact,
        display_name: None,
    });
    let text = (0..(super::types::MAX_RESULT_BLOCKS - super::types::MAX_RESULT_FILES))
        .map(|_| ToolResultBlock::Text { text: String::new() });
    let blocks = files.chain(text).collect();

    assert!(validate(&ToolResultContent::Blocks(blocks)).is_ok());
}

#[test]
fn serde_accepts_camel_case_display_name_and_rejects_closed_forms() {
    let valid: ToolResultContent = serde_json::from_value(serde_json::json!([{"type":"file","path":"out.txt","purpose":"artifact","displayName":"Output"}])).unwrap();
    assert!(validate(&valid).is_ok());
    for value in [serde_json::json!([{"type":"file","path":"out","purpose":"other"}]), serde_json::json!([{"type":"text","text":"x","extra":true}])] {
        assert!(serde_json::from_value::<ToolResultContent>(value).is_err());
    }
}

#[test]
fn rejects_empty_or_control_display_names_after_deserialization() {
    for display_name in ["", "report\nname"] {
        let content: ToolResultContent = serde_json::from_value(serde_json::json!([
            {"type":"file","path":"out.txt","purpose":"artifact","displayName":display_name}
        ]))
        .expect("shape remains valid before semantic validation");
        assert!(validate(&content).is_err());
    }
}

#[test]
fn checked_accumulation_rejects_limits_and_overflow() {
    assert_eq!(super::tool_result_contract::checked_add(2, 3, 5), Ok(5));
    assert!(super::tool_result_contract::checked_add(5, 1, 5).is_err());
    assert!(super::tool_result_contract::checked_add(usize::MAX, 1, usize::MAX).is_err());
}
