#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CefProcessRole {
    Helper = 1,
}

impl TryFrom<u8> for CefProcessRole {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Helper),
            _ => Err(()),
        }
    }
}

impl From<CefProcessRole> for u8 {
    fn from(value: CefProcessRole) -> Self {
        value as Self
    }
}

const MAX_NATIVE_PROCESSES: usize = 4_096;
const MAX_DEDICATED_WEBVIEWS: usize = 64;
const MAX_ANCESTRY_DEPTH: usize = 32;
const MAX_PROCESS_NAME_CHARS: usize = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NativeProcessRecord {
    pub(super) pid: u32,
    pub(super) parent_pid: u32,
    pub(super) name: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NativeWebViewRole {
    Dedicated,
    SharedSystem,
    Other,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NativeWebViewObservation {
    pub dedicated_pids: Vec<u32>,
    pub shared_system_count: usize,
}

pub(super) fn classify_native_webview(
    platform: &str,
    root_pid: u32,
    candidate_pid: u32,
    records: &[NativeProcessRecord],
) -> NativeWebViewRole {
    let Some(candidate) = records.iter().find(|record| record.pid == candidate_pid) else {
        return NativeWebViewRole::Other;
    };
    let name = candidate.name.to_ascii_lowercase();
    if platform == "macos" && is_macos_shared_webkit(&name) {
        return NativeWebViewRole::SharedSystem;
    }
    let dedicated_name = match platform {
        "windows" => name == "msedgewebview2.exe",
        "linux" => is_linux_webkit(&name),
        _ => false,
    };
    if dedicated_name && is_descendant(root_pid, candidate_pid, records) {
        NativeWebViewRole::Dedicated
    } else {
        NativeWebViewRole::Other
    }
}

pub(crate) fn observe_native_webviews() -> NativeWebViewObservation {
    let root_pid = std::process::id();
    let mut system = sysinfo::System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let records = system
        .processes()
        .iter()
        .take(MAX_NATIVE_PROCESSES)
        .map(|(pid, process)| NativeProcessRecord {
            pid: pid.as_u32(),
            parent_pid: process.parent().map(sysinfo::Pid::as_u32).unwrap_or(0),
            name: process
                .name()
                .to_string_lossy()
                .chars()
                .take(MAX_PROCESS_NAME_CHARS)
                .collect(),
        })
        .collect::<Vec<_>>();
    let mut dedicated_pids = Vec::with_capacity(MAX_DEDICATED_WEBVIEWS);
    let mut shared_system_count = 0_usize;
    for record in &records {
        match classify_native_webview(std::env::consts::OS, root_pid, record.pid, &records) {
            NativeWebViewRole::Dedicated if dedicated_pids.len() < MAX_DEDICATED_WEBVIEWS => {
                dedicated_pids.push(record.pid);
            }
            NativeWebViewRole::SharedSystem => {
                shared_system_count = shared_system_count.saturating_add(1);
            }
            _ => {}
        }
    }
    NativeWebViewObservation {
        dedicated_pids,
        shared_system_count,
    }
}

fn is_descendant(root_pid: u32, candidate_pid: u32, records: &[NativeProcessRecord]) -> bool {
    if root_pid < 2 || candidate_pid < 2 || root_pid == candidate_pid {
        return false;
    }
    let mut current = candidate_pid;
    for _ in 0..MAX_ANCESTRY_DEPTH {
        let Some(record) = records.iter().find(|record| record.pid == current) else {
            return false;
        };
        if record.parent_pid == root_pid {
            return true;
        }
        if record.parent_pid < 2 || record.parent_pid == current {
            return false;
        }
        current = record.parent_pid;
    }
    false
}

fn is_linux_webkit(name: &str) -> bool {
    name.starts_with("webkitweb")
        || name.starts_with("webkitnetwork")
        || name.starts_with("webkitgpu")
}

fn is_macos_shared_webkit(name: &str) -> bool {
    name.starts_with("com.apple.webkit.") || name.starts_with("webkitwebcontent")
}

#[cfg(any(test, target_os = "macos"))]
pub(super) fn validate_browser_process_result(result: i32) -> Result<(), ()> {
    (result == -1).then_some(()).ok_or(())
}
