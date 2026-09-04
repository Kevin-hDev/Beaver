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
            | "load_skill"
            | "read_spreadsheet"
            | "read_document"
    ) || name == crate::services::extensions::LIST_EXTENSIONS_TOOL_NAME
        || name == super::tool_extension_resource::NAME
}

#[cfg(test)]
mod tests {
    #[test]
    fn resource_loading_is_read_only_but_inspection_is_not() {
        assert!(super::is_read_only("list_extensions"));
        assert!(!super::is_read_only("inspect_extensions"));
        assert!(super::is_read_only(super::super::tool_extension_resource::NAME));
    }
}
