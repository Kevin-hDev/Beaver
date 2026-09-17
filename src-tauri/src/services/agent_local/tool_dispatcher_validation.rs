pub(super) fn validate(
    dynamic_tool: bool,
    tool_name: &str,
    args: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    if dynamic_tool {
        crate::services::extensions::validate_arguments(tool_name, args)
    } else {
        super::tool_validate::validate(tool_name, args)
    }
}
