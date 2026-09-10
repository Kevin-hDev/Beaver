use super::max_lines;

#[test]
fn legacy_log_facade_uses_the_canonical_history_limit() {
    assert_eq!(max_lines(), 500);
}
