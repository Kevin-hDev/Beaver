use std::collections::VecDeque;
use std::path::Path;
use std::process::Stdio;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::OnceCell;
use zeroize::{Zeroize, Zeroizing};

#[path = "tool_bash_profile_sanitize.rs"]
mod sanitize;

const MAX_CACHED_PROFILES: usize = 64;
const MAX_SNAPSHOT_BYTES: usize = 128 * 1024;
const SNAPSHOT_TIMEOUT_SECS: u64 = 5;
pub(super) const SNAPSHOT_ENVS: [&str; 2] = [
    "BEAVER_INTERNAL_PROFILE_SNAPSHOT_0",
    "BEAVER_INTERNAL_PROFILE_SNAPSHOT_1",
];

type CachedProfile = Option<Arc<ShellProfile>>;
type ProfileCell = Arc<OnceCell<CachedProfile>>;

static CACHE: LazyLock<Mutex<VecDeque<(String, ProfileCell)>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

pub struct ShellProfile {
    scripts: [Zeroizing<String>; 2],
}

impl ShellProfile {
    pub fn apply(&self, command: &mut Command) {
        for (name, script) in SNAPSHOT_ENVS.iter().zip(&self.scripts) {
            command.env(name, script.as_str());
        }
    }
}

pub async fn prepare(
    owner_session_id: &str,
    shell: &str,
    working_dir: &Path,
) -> Option<Arc<ShellProfile>> {
    let cell = profile_cell(owner_session_id)?;
    let shell = shell.to_string();
    let working_dir = working_dir.to_path_buf();
    let profile = cell
        .get_or_init(|| async move { capture(&shell, &working_dir).await.map(Arc::new) })
        .await;
    profile.clone()
}

pub fn clear() {
    CACHE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clear();
}

fn profile_cell(owner_session_id: &str) -> Option<ProfileCell> {
    super::session_store::validate_session_id(owner_session_id).ok()?;
    let mut cache = CACHE.lock().unwrap_or_else(|error| error.into_inner());
    if let Some(position) = cache.iter().position(|(id, _)| id == owner_session_id) {
        let entry = cache.remove(position)?;
        let cell = Arc::clone(&entry.1);
        cache.push_back(entry);
        return Some(cell);
    }
    if cache.len() >= MAX_CACHED_PROFILES {
        cache.pop_front();
    }
    let cell = Arc::new(OnceCell::new());
    cache.push_back((owner_session_id.to_string(), Arc::clone(&cell)));
    Some(cell)
}

async fn capture(shell: &str, working_dir: &Path) -> Option<ShellProfile> {
    let marker = format!("__BEAVER_PROFILE_{}__", uuid::Uuid::new_v4().simple());
    let script = snapshot_script(shell, &marker)?;
    let arguments = vec!["-l".to_string(), "-c".to_string(), script];
    let prepared = super::shell_sandbox::prepare_command(
        std::ffi::OsStr::new(shell),
        &arguments,
        working_dir,
    )
    .ok()?;
    let cleanup_dir = prepared.cleanup_dir;
    let mut command = prepared.command;
    command
        .current_dir(working_dir)
        .env("SHELL", shell)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    super::tool_bash_platform::configure_process_group(&mut command);
    let result = capture_process(command, &marker).await;
    super::shell_sandbox::cleanup_temp(cleanup_dir).await;
    result
}

async fn capture_process(
    mut command: tokio::process::Command,
    marker: &str,
) -> Option<ShellProfile> {
    let mut child = command.spawn().ok()?;
    let Some(pid) = child.id() else {
        let _ = child.kill().await;
        let _ = child.wait().await;
        return None;
    };
    let Some(stdout) = child.stdout.take() else {
        super::tool_bash_platform::terminate_process_tree(pid).await;
        let _ = child.wait().await;
        return None;
    };
    let capture = tokio::time::timeout(
        Duration::from_secs(SNAPSHOT_TIMEOUT_SECS),
        async {
            tokio::join!(
                super::tool_bash_io::read_bounded(stdout, MAX_SNAPSHOT_BYTES),
                child.wait()
            )
        },
    )
    .await;
    let (mut bytes, exceeded, status) = match capture {
        Ok((Ok((bytes, exceeded)), Ok(status))) => (bytes, exceeded, status),
        Ok((mut output, _)) => {
            if let Ok((bytes, _)) = &mut output {
                bytes.zeroize();
            }
            super::tool_bash_platform::terminate_process_tree(pid).await;
            let _ = child.wait().await;
            return None;
        }
        Err(_) => {
            super::tool_bash_platform::terminate_process_tree(pid).await;
            let _ = child.wait().await;
            return None;
        }
    };
    if exceeded || !status.success() {
        bytes.zeroize();
        log_unavailable(if exceeded { "too_large" } else { "capture_failed" });
        return None;
    }
    let raw = Zeroizing::new(String::from_utf8_lossy(&bytes).into_owned());
    bytes.zeroize();
    let start = raw.find(marker)? + marker.len();
    let snapshot = Zeroizing::new(sanitize::snapshot(
        raw[start..].trim_start_matches(['\r', '\n']),
    ));
    if snapshot.is_empty() || snapshot.contains('\0') {
        return None;
    }
    Some(ShellProfile {
        scripts: sanitize::chunks(&snapshot),
    })
}

fn log_unavailable(reason: &str) {
    eprintln!("[shell-profile] snapshot unavailable: {reason}");
}

fn snapshot_script(shell: &str, marker: &str) -> Option<String> {
    let name = Path::new(shell).file_name()?.to_string_lossy();
    if name == "zsh" {
        Some(format!(
            "rc=${{ZDOTDIR:-$HOME}}/.zshrc; [[ -r \"$rc\" ]] && . \"$rc\" </dev/null; print -r -- '{marker}'; print -r -- 'unalias -a 2>/dev/null || true'; functions; setopt | sed 's/^/setopt /'; alias -L; export -p"
        ))
    } else if name == "bash" {
        Some(format!(
            "[[ -r \"$HOME/.bashrc\" ]] && . \"$HOME/.bashrc\" </dev/null; printf '%s\\n' '{marker}'; printf '%s\\n' 'unalias -a 2>/dev/null || true' 'shopt -s expand_aliases 2>/dev/null || true'; declare -f; set -o | awk '$2==\"on\"{{print \"set -o \"$1}}'; alias -p; export -p"
        ))
    } else if matches!(name.as_ref(), "sh" | "dash" | "ksh" | "ksh93") {
        Some(format!(
            "printf '%s\\n' '{marker}'; printf '%s\\n' 'unalias -a 2>/dev/null || true'; (typeset -f 2>/dev/null || true); alias 2>/dev/null || true; export -p"
        ))
    } else {
        None
    }
}

pub(super) fn supports_shell(shell: &str) -> bool {
    snapshot_script(shell, "supported").is_some()
}

#[cfg(test)]
#[path = "tool_bash_profile_tests.rs"]
mod tests;
