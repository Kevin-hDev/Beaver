use super::DelegateBatchOutput;
use crate::services::agent_local::types_tools::ToolResult;

#[test]
fn keeps_parent_tool_context_in_original_order() {
    let mut outputs = vec![
        DelegateBatchOutput {
            index: 2,
            result: ToolResult::ok("third"),
        },
        DelegateBatchOutput {
            index: 0,
            result: ToolResult::ok("first"),
        },
        DelegateBatchOutput {
            index: 1,
            result: ToolResult::ok("second"),
        },
    ];

    outputs.sort_by_key(|output| output.index);

    let contents = outputs
        .into_iter()
        .map(|output| output.result.content)
        .collect::<Vec<_>>();
    assert_eq!(contents, ["first", "second", "third"]);
}
