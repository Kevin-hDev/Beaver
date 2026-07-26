#[cfg(test)]
mod tests {
    use crate::services::agent_local::tool_executor_read_only::is_read_only;

    #[test]
    fn read_only_classification() {
        assert!(is_read_only("read_file"));
        assert!(is_read_only("grep"));
        assert!(is_read_only("glob"));
        assert!(is_read_only("list_dir"));
        assert!(is_read_only("web_search"));
        assert!(is_read_only("read_spreadsheet"));
        assert!(is_read_only("read_document"));
        assert!(is_read_only("read_image"));
        assert!(!is_read_only("bash"));
        assert!(!is_read_only("write_file"));
        assert!(!is_read_only("edit_file"));
        assert!(!is_read_only("web_fetch"));
        assert!(!is_read_only("write_spreadsheet"));
        assert!(!is_read_only("write_document"));
        assert!(!is_read_only("process_image"));
    }
}
