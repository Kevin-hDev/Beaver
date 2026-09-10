use sysinfo::{ProcessesToUpdate, System};

const MAX_PROCESSES: usize = 4096;

pub fn matches_app_process(name: &str) -> bool {
    name == "cl-go-dash" || name.eq_ignore_ascii_case("cl-go-dash.exe")
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
    let current_pid = std::process::id();
    process_exists_in(
        system
            .processes()
            .iter()
            .map(|(pid, process)| (pid.as_u32(), process.name())),
        current_pid,
        matches,
        fail_closed,
    )
}

fn process_exists_in<'a>(
    processes: impl IntoIterator<Item = (u32, &'a std::ffi::OsStr)>,
    current_pid: u32,
    matches: fn(&str) -> bool,
    fail_closed: bool,
) -> bool {
    let mut count = 0_usize;
    let mut current_seen = false;
    let mut matched = false;
    for (pid, name) in processes {
        count += 1;
        if count > MAX_PROCESSES {
            return fail_closed;
        }
        current_seen |= pid == current_pid;
        matched |= pid != current_pid && matches(&name.to_string_lossy());
    }
    if !current_seen {
        return fail_closed;
    }
    matched
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correspond_au_binaire_app_exact() {
        assert!(matches_app_process("cl-go-dash"));
        assert!(matches_app_process("cl-go-dash.exe"));
        assert!(!matches_app_process("cl-go-dash-updater"));
        assert!(!matches_app_process("cl-go-dash-updater.exe"));
        assert!(!matches_app_process("cl-go-dash-helper"));
        assert!(!matches_app_process("beaver"));
    }

    #[test]
    fn instantane_incomplet_bloque_seulement_les_actions_destructrices() {
        let processes = [(41_u32, std::ffi::OsStr::new("other"))];

        assert!(process_exists_in(processes, 42, matches_app_process, true));
        assert!(!process_exists_in(
            processes,
            42,
            matches_ollama_process,
            false
        ));
    }

    #[test]
    fn instantane_sain_exclut_la_cli_et_detecte_l_app() {
        let processes = [
            (42_u32, std::ffi::OsStr::new("beaver")),
            (43_u32, std::ffi::OsStr::new("cl-go-dash")),
        ];

        assert!(process_exists_in(processes, 42, matches_app_process, true));
    }

    #[test]
    fn correspond_au_processus_ollama_exact() {
        assert!(matches_ollama_process("ollama"));
        assert!(matches_ollama_process("ollama.exe"));
        assert!(matches_ollama_process(".beaver-gated-19787-d1QCQW"));
        assert!(!matches_ollama_process("ollama-helper"));
    }
}
