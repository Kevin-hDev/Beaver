pub fn available(
    enabled_by_settings: bool,
    dynamic: bool,
    replacement: bool,
) -> bool {
    enabled_by_settings || (dynamic && !replacement)
}

#[cfg(test)]
mod tests {
    use super::available;

    #[test]
    fn replacement_respects_a_disabled_core_tool_setting() {
        assert!(!available(false, true, true));
    }

    #[test]
    fn a_new_extension_tool_does_not_need_a_core_setting() {
        assert!(available(false, true, false));
    }
}
