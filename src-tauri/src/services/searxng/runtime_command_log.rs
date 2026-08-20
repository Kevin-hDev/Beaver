use std::fs;
use std::sync::Mutex;

use super::runtime_command::RuntimeCommandError;

const MAX_LOG_BYTES: usize = 16_384;
const MAX_RENDERED_STREAM_BYTES: usize = 4_096;
static LOG_WRITE: Mutex<()> = Mutex::new(());

pub(super) fn write(
    error: RuntimeCommandError,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
) -> Result<(), ()> {
    let _guard = LOG_WRITE.lock().map_err(|_| ())?;
    let body = format!(
        "stage={}\ncategory={}\nexit_code={}\nstdout_tail={}\nstderr_tail={}\n",
        error.stage().as_str(),
        error.category(),
        error
            .exit_code()
            .map_or_else(|| "none".to_string(), |code| code.to_string()),
        render(&stdout),
        render(&stderr),
    );
    if body.len() > MAX_LOG_BYTES {
        return Err(());
    }
    let path = super::paths::runtime_log_path();
    if let Some(parent) = path.parent() {
        if parent.exists()
            && !fs::symlink_metadata(parent)
                .map_err(|_| ())?
                .file_type()
                .is_dir()
        {
            return Err(());
        }
    } else {
        return Err(());
    }
    reject_link(&path)?;
    crate::services::private_store::atomic_write(&path, body.as_bytes()).map_err(|_| ())
}

fn reject_link(path: &std::path::Path) -> Result<(), ()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(()),
        Ok(metadata) if metadata.file_type().is_file() && single_link(&metadata) => Ok(()),
        Ok(_) => Err(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(()),
    }
}

#[cfg(unix)]
fn single_link(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    metadata.nlink() == 1
}

#[cfg(not(unix))]
fn single_link(_: &fs::Metadata) -> bool {
    true
}

fn render(bytes: &[u8]) -> String {
    let normalized: String = String::from_utf8_lossy(bytes)
        .chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .collect();
    let mut rendered = String::with_capacity(MAX_RENDERED_STREAM_BYTES);
    let mut redact_next = false;
    for token in normalized.split_whitespace() {
        let sensitive = redact_next || sensitive_marker(token);
        redact_next = sensitive_marker(token);
        let token = if sensitive { "[redacted]" } else { token };
        if !rendered.is_empty() {
            if rendered.len() == MAX_RENDERED_STREAM_BYTES {
                return rendered;
            }
            rendered.push(' ');
        }
        for character in token.chars() {
            if rendered.len() + character.len_utf8() > MAX_RENDERED_STREAM_BYTES {
                return rendered;
            }
            rendered.push(character);
        }
    }
    rendered
}

fn sensitive_marker(token: &str) -> bool {
    let lower = token.to_ascii_lowercase();
    lower.contains("://")
        || lower.contains("token")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("authori")
        || lower.contains("bear")
        || lower.contains("credential")
        || lower.contains("api_key")
        || lower.contains("api-key")
        || lower.contains("apikey")
        || lower.contains("access-key")
        || lower.starts_with("sk-")
}
