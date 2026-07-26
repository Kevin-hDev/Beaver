use super::cef_diagnostics::diagnostic_line;
use cef::TerminationStatus;

#[test]
fn renderer_diagnostic_is_bounded_and_contains_no_external_text() {
    let line = diagnostic_line(TerminationStatus::PROCESS_CRASHED, i32::MIN);

    assert!(line.len() < 128);
    assert!(line.starts_with("[browser] renderer terminated (status="));
    assert!(line.ends_with(&format!("code={})", i32::MIN)));
}
