pub fn is_read_only(name: &str) -> bool {
    if let Some(indexed) = crate::services::extensions::indexed_tool(name) {
        return super::permission_policy::extension_effect_policy(indexed.tool.effect)
            .parallel_read;
    }
    matches!(
        name,
        "read_file"
            | "grep"
            | "glob"
            | "list_dir"
            | "web_search"
            | "search_extension_tools"
            | "load_skill"
            | "read_spreadsheet"
            | "read_document"
    )
}
