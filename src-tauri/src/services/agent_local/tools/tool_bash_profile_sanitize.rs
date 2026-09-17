const SANDBOX_OWNED_ENVS: [&str; 5] = ["PATH", "TMPDIR", "TMP", "TEMP", "TMPPREFIX"];
pub(super) const MAX_SNAPSHOT_CHUNK_BYTES: usize = 64 * 1024;

pub(super) fn snapshot(snapshot: &str) -> String {
    let mut sanitized = String::with_capacity(snapshot.len());
    for line in snapshot.lines() {
        if rejected_export(line) {
            continue;
        }
        sanitized.push_str(line);
        sanitized.push('\n');
    }
    sanitized
}

pub(super) fn chunks(snapshot: &str) -> [zeroize::Zeroizing<String>; 2] {
    let mut split_at = snapshot.len().min(MAX_SNAPSHOT_CHUNK_BYTES);
    while !snapshot.is_char_boundary(split_at) {
        split_at = split_at.saturating_sub(1);
    }
    [
        zeroize::Zeroizing::new(snapshot[..split_at].to_string()),
        zeroize::Zeroizing::new(snapshot[split_at..].to_string()),
    ]
}

fn rejected_export(line: &str) -> bool {
    let line = line.trim_start();
    for prefix in ["export ", "declare -x ", "typeset -x "] {
        let Some(value) = line.strip_prefix(prefix) else {
            continue;
        };
        let Some((name, _)) = value.split_once('=') else { return true };
        let name = name.trim();
        return SANDBOX_OWNED_ENVS.contains(&name)
            || super::super::shell_sandbox::is_process_injection_env(name)
            || name.starts_with("BEAVER_INTERNAL_");
    }
    false
}
