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
    match crate::services::private_store::read_bounded_regular(path, MAX_LOG_BYTES as u64)
        .map_err(|_| ())?
    {
        crate::services::private_store::BoundedFile::Missing
        | crate::services::private_store::BoundedFile::Content(_) => Ok(()),
    }
}

fn render(bytes: &[u8]) -> String {
    let normalized: String = String::from_utf8_lossy(bytes)
        .chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .collect();
    let mut rendered = String::with_capacity(MAX_RENDERED_STREAM_BYTES);
    let mut redact_values = 0_u8;
    for token in normalized.split_whitespace() {
        let marker = sensitive_marker(token);
        let separator = token.chars().all(|character| !character.is_alphanumeric());
        let sensitive = marker || redact_values > 0;
        if marker {
            redact_values = 1;
        } else if redact_values > 0 && !separator {
            redact_values -= 1;
        }
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
