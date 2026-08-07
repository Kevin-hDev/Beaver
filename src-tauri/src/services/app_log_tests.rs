use super::app_log::{format_message, format_record};
use chrono::{TimeZone, Utc};

#[test]
fn log_messages_are_redacted_and_bounded() {
    let long = "x".repeat(4_000);
    let message = format_args!("failed at /Users/private/project token=abc123 sk-secret {long}");
    let output = format_message(&message);

    assert!(!output.contains("/Users/private"));
    assert!(!output.contains("abc123"));
    assert!(!output.contains("sk-secret"));
    assert!(output.chars().count() <= 2_048);
}

#[test]
fn log_messages_cannot_inject_additional_lines() {
    let output = format_message(&format_args!("first\nsecond\rthird"));
    assert!(!output.contains(['\n', '\r']));
}

#[test]
fn secrets_crossing_the_output_limit_are_redacted_before_truncation() {
    let payload = format!("{}sk-secret-value", "x".repeat(2_045));
    let output = format_message(&format_args!("{payload}"));

    assert!(!output.contains("sk-"));
    assert!(output.contains("[redacted]"));
}

#[test]
fn formatted_records_include_a_utc_timestamp() {
    let timestamp = Utc.with_ymd_and_hms(2026, 8, 7, 12, 34, 56).unwrap();
    let output = format_record(
        timestamp,
        log::Level::Info,
        "beaver::test",
        &format_args!("ready"),
    );

    assert_eq!(
        output,
        "[2026-08-07T12:34:56.000Z][INFO][beaver::test] ready"
    );
}

fn assert_log_level(source: &str, message: &str, level: &str) {
    let lines: Vec<_> = source.lines().collect();
    let line = lines
        .iter()
        .position(|candidate| candidate.contains(message))
        .expect("reviewed log message");
    let window = lines[line.saturating_sub(2)..=line].join("\n");
    assert!(
        window.contains(&format!("::log::{level}!")),
        "{message} must use {level}"
    );
}

#[test]
fn reviewed_log_events_use_meaningful_levels() {
    let lifecycle = include_str!("ollama_lifecycle.rs");
    assert_log_level(lifecycle, "[ollama] GPU", "info");
    assert_log_level(lifecycle, "[ollama] env", "info");
    assert_log_level(lifecycle, "LD_LIBRARY_PATH prépend", "info");
    assert_log_level(lifecycle, "[ollama] spawn", "error");

    let startup = include_str!("../lib.rs");
    assert_log_level(startup, "policy unavailable", "error");
    assert_log_level(startup, "[vault] init failed", "error");

    let automatic = include_str!("llm/compress_hook.rs");
    assert_log_level(automatic, "auto llm start", "info");
    assert_log_level(automatic, "auto llm done", "info");

    let manual = include_str!("../commands/agent_chat_task/compress.rs");
    assert_log_level(manual, "manual start", "info");
    assert_log_level(manual, "manual done", "info");

    let cleanup = include_str!("agent_local/subagent_startup_cleanup.rs");
    assert_log_level(cleanup, "sous-agent(s) orphelin(s) nettoyé(s)", "info");
}
