pub fn is_read_only(name: &str) -> bool {
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
