pub fn dynamic_route(
    registered_dynamic: bool,
    active_dynamic: bool,
    replacement: bool,
) -> Result<bool, &'static str> {
    if active_dynamic {
        Ok(true)
    } else if registered_dynamic && !replacement {
        Err("Extension indisponible.")
    } else {
        Ok(false)
    }
}

pub fn is_chat_tool(tool_name: &str) -> bool {
    matches!(tool_name, "web_search" | "web_fetch")
}
