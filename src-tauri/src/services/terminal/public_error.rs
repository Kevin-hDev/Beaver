pub(super) fn map_admission_error(code: &str) -> String {
    if code == "app-shutting-down" || code == "service-shutting-down" {
        shutting_down()
    } else {
        "terminal-capacity-reached".to_string()
    }
}

pub(super) fn shutting_down() -> String {
    "terminal-shutting-down".to_string()
}

pub(super) fn not_found() -> String {
    "terminal-not-found".to_string()
}

pub(crate) fn terminal_error() -> String {
    "terminal-error".to_string()
}
