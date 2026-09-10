use sysinfo::{ProcessesToUpdate, System};

const MAX_PROCESSES: usize = 4096;

pub fn matches_app_process(name: &str) -> bool {
    name == "cl-go-dash"
}

pub fn matches_ollama_process(name: &str) -> bool {
    cl_go_dash_lib::cli_support::ollama_process_name_matches(name)
}

pub fn app_is_running() -> bool {
    process_exists(matches_app_process, true)
}

pub fn ollama_process_running() -> bool {
    process_exists(matches_ollama_process, false)
}

fn process_exists(matches: fn(&str) -> bool, fail_closed: bool) -> bool {
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);
    if system.processes().len() > MAX_PROCESSES {
        return fail_closed;
    }
    let current_pid = std::process::id();
    system.processes().iter().any(|(pid, process)| {
        pid.as_u32() != current_pid && matches(&process.name().to_string_lossy())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correspond_au_binaire_app_exact() {
        assert!(matches_app_process("cl-go-dash"));
        assert!(!matches_app_process("cl-go-dash-updater"));
        assert!(!matches_app_process("cl-go-dash-helper"));
        assert!(!matches_app_process("beaver"));
    }

    #[test]
    fn correspond_au_processus_ollama_exact() {
        assert!(matches_ollama_process("ollama"));
        assert!(matches_ollama_process("ollama.exe"));
        assert!(matches_ollama_process(".beaver-gated-19787-d1QCQW"));
        assert!(!matches_ollama_process("ollama-helper"));
    }
}
