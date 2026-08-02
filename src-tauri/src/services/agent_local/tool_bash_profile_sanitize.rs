const SANDBOX_TEMP_ENVS: [&str; 4] = ["TMPDIR", "TMP", "TEMP", "TMPPREFIX"];
pub(super) const MAX_SNAPSHOT_CHUNK_BYTES: usize = 64 * 1024;

pub(super) fn snapshot(snapshot: &str) -> String {
    let mut sanitized = String::with_capacity(snapshot.len());
    for line in snapshot.lines() {
        if overrides_sandbox_temp(line) {
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

fn overrides_sandbox_temp(line: &str) -> bool {
    let line = line.trim_start();
    for prefix in ["export ", "declare -x ", "typeset -x "] {
        let Some(value) = line.strip_prefix(prefix) else {
            continue;
        };
        if SANDBOX_TEMP_ENVS.iter().any(|name| {
            value
                .strip_prefix(name)
                .is_some_and(|value| value.starts_with('='))
        }) {
            return true;
        }
    }
    false
}
