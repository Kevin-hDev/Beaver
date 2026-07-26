use super::process_role::validate_browser_process_result;

#[test]
fn only_the_browser_process_result_can_continue_initialization() {
    assert!(validate_browser_process_result(-1).is_ok());
    assert!(validate_browser_process_result(0).is_err());
    assert!(validate_browser_process_result(7).is_err());
}
