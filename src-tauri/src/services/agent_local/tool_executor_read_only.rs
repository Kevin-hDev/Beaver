pub fn is_read_only(name: &str) -> bool {
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
            | "read_image"
    )
}
