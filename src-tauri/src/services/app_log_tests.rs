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
