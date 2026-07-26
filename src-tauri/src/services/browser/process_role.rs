pub(super) fn validate_browser_process_result(result: i32) -> Result<(), ()> {
    (result == -1).then_some(()).ok_or(())
}
