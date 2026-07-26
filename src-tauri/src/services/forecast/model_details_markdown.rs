pub fn is_relative_target(target: &str) -> bool {
    !target.is_empty()
        && !target.starts_with("http://")
        && !target.starts_with("https://")
        && !target.starts_with("data:")
        && !target.starts_with('#')
        && !target.starts_with("mailto:")
}

pub fn normalize_relative_target(target: &str) -> &str {
    target
        .strip_prefix("./")
        .or_else(|| target.strip_prefix('/'))
        .unwrap_or(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_local_targets_are_rewritten() {
        assert!(is_relative_target("./assets/chart.png"));
        assert!(!is_relative_target("https://example.com/chart.png"));
        assert!(!is_relative_target("#section"));
    }

    #[test]
    fn relative_prefix_is_removed() {
        assert_eq!(normalize_relative_target("./README.md"), "README.md");
        assert_eq!(normalize_relative_target("/README.md"), "README.md");
    }
}
