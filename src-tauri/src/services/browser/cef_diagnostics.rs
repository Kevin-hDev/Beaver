use cef::TerminationStatus;

pub(super) fn log_renderer_termination(status: TerminationStatus, error_code: i32) {
    #[cfg(debug_assertions)]
    ::log::warn!("{}", diagnostic_line(status, error_code));
    #[cfg(not(debug_assertions))]
    let _ = (status, error_code);
}

#[cfg(any(test, debug_assertions))]
pub(super) fn diagnostic_line(status: TerminationStatus, error_code: i32) -> String {
    format!(
        "[browser] renderer terminated (status={}, code={error_code})",
        status.get_raw()
    )
}
