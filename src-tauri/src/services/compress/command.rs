const EXPLICIT_COMPRESSION_COMMAND: &str = "/compress";

pub fn is_explicit_compression_command(content: &str) -> bool {
    content.trim() == EXPLICIT_COMPRESSION_COMMAND
}

#[cfg(test)]
mod tests {
    #[test]
    fn command_allows_surrounding_whitespace_but_not_extra_content() {
        assert!(super::is_explicit_compression_command("  /compress\n"));
        assert!(!super::is_explicit_compression_command("/compress now"));
    }
}
