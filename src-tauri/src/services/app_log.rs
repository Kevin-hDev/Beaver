use std::{
    fmt::{Arguments, Write},
    future::Future,
};

use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

const MAX_LOG_CHARS: usize = 2_048;
const MAX_REDACTION_INPUT_CHARS: usize = MAX_LOG_CHARS * 4;
const MAX_FILE_BYTES: u128 = 2 * 1024 * 1024;
const RETAINED_FILES: usize = 4;
const WINDOWS_TLS_VERIFIER_TARGET: &str = "rustls_platform_verifier::verification::windows";

tokio::task_local! {
    static EXPECTED_LOCAL_TLS_REJECTION: ();
}

struct BoundedMessage {
    value: String,
    remaining: usize,
}

impl BoundedMessage {
    fn new(limit: usize) -> Self {
        Self {
            value: String::with_capacity(limit),
            remaining: limit,
        }
    }
}

impl Write for BoundedMessage {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        if self.remaining == 0 {
            return Ok(());
        }
        let part: String = value.chars().take(self.remaining).collect();
        self.remaining = self.remaining.saturating_sub(part.chars().count());
        self.value.push_str(&part);
        Ok(())
    }
}

pub(crate) fn format_message(message: &Arguments<'_>) -> String {
    let mut bounded = BoundedMessage::new(MAX_REDACTION_INPUT_CHARS);
    let _ = bounded.write_fmt(*message);
    crate::services::agent_local::diagnostic_redaction::redact_text(&bounded.value)
        .replace(['\n', '\r', '\t'], " ")
        .chars()
        .take(MAX_LOG_CHARS)
        .collect()
}

pub(crate) fn format_record(
    timestamp: chrono::DateTime<chrono::Utc>,
    level: log::Level,
    target: &str,
    message: &Arguments<'_>,
) -> String {
    let timestamp = timestamp.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    format!(
        "[{timestamp}][{level}][{target}] {}",
        format_message(message)
    )
}

pub(crate) fn should_emit(metadata: &log::Metadata<'_>) -> bool {
    metadata.target() != WINDOWS_TLS_VERIFIER_TARGET
        || EXPECTED_LOCAL_TLS_REJECTION.try_with(|_| ()).is_err()
}

pub(crate) async fn with_expected_local_tls_rejection<F>(future: F) -> F::Output
where
    F: Future,
{
    // A rejected local certificate is an expected discovery result; the scope
    // keeps every verifier error from unrelated concurrent requests visible.
    EXPECTED_LOCAL_TLS_REJECTION.scope((), future).await
}

pub fn plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    let logs = crate::services::paths::data_dir().join("logs");
    tauri_plugin_log::Builder::new()
        .targets([
            Target::new(TargetKind::Stderr),
            Target::new(TargetKind::Folder {
                path: logs,
                file_name: Some("beaver".to_string()),
            }),
        ])
        .level(log::LevelFilter::Info)
        .filter(should_emit)
        .max_file_size(MAX_FILE_BYTES)
        .rotation_strategy(RotationStrategy::KeepSome(RETAINED_FILES))
        .format(|out, message, record| {
            let safe = format_record(chrono::Utc::now(), record.level(), record.target(), message);
            out.finish(format_args!("{safe}"));
        })
        .build()
}
