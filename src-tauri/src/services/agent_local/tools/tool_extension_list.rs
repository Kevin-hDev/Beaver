use super::types_tools::ToolResult;

pub async fn execute() -> ToolResult {
    execute_with(|| {
        let entries = crate::services::extensions::list_discoverable(
            &crate::services::extensions::indexed_plugins(),
        )?;
        crate::services::extensions::serialize_bounded_result(&entries)
    })
}

fn execute_with(load: impl FnOnce() -> Result<String, ()>) -> ToolResult {
    match load() {
        Ok(content) => ToolResult::ok(content),
        Err(_) => unavailable_result(),
    }
}

fn unavailable_result() -> ToolResult {
    ToolResult::unavailable(
        crate::services::extensions::error_codes::LISTING_UNAVAILABLE,
        "Extensions indisponibles.",
        true,
    )
}

#[cfg(test)]
#[path = "tool_extension_list_tests.rs"]
mod tests;
