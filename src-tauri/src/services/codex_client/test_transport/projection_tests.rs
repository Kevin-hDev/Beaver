use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::super::sensitive_buffer::SensitiveBuffer;

#[test]
fn nested_sensitive_fields_are_detected_without_retaining_their_values() {
    for (field, secret, location) in [
        ("access_token", "secret-access-value", "input"),
        ("refresh_token", "secret-refresh-value", "tools"),
        ("authorization", "secret-authorization-value", "tools"),
    ] {
        let body = format!(
            r#"{{"model":"gpt-5.6-sol","input":[{{"content":{{{input}}}}}],"tools":[{{"schema":[{{{tools}}}]}}]}}"#,
            input = if location == "input" {
                format!(r#""{field}":"{secret}""#)
            } else {
                r#""safe":"value""#.to_string()
            },
            tools = if location == "tools" {
                format!(r#""{field}":"{secret}""#)
            } else {
                r#""safe":"value""#.to_string()
            },
        );
        let zeroized = Arc::new(AtomicBool::new(false));
        let mut buffer = SensitiveBuffer::with_capacity(body.len(), Arc::clone(&zeroized));
        buffer.extend_from_slice(body.as_bytes());

        let projection = super::parse(&buffer).expect("bounded nested request parses");

        assert!(projection.forbidden_field_present, "field: {field}");
        assert!(!format!("{projection:?}").contains(secret));
        drop(buffer);
        assert!(zeroized.load(Ordering::SeqCst));
    }
}

#[test]
fn nesting_beyond_32_levels_fails_closed() {
    let nested = format!("{}null{}", "[".repeat(33), "]".repeat(33));
    let body = format!(r#"{{"model":"gpt-5.6-sol","input":[],"tools":[],"metadata":{nested}}}"#);

    assert_eq!(
        super::parse(body.as_bytes()).expect_err("depth is bounded"),
        super::invalid()
    );
}

#[test]
fn more_than_16384_total_elements_fails_closed() {
    let elements = vec!["null"; 16_385].join(",");
    let body =
        format!(r#"{{"model":"gpt-5.6-sol","input":[],"tools":[],"metadata":[{elements}]}}"#);

    assert_eq!(
        super::parse(body.as_bytes()).expect_err("total elements are bounded"),
        super::invalid()
    );
}

#[test]
fn nested_key_longer_than_256_bytes_fails_closed() {
    let key = "k".repeat(257);
    let body = format!(r#"{{"model":"gpt-5.6-sol","input":[{{"{key}":null}}],"tools":[]}}"#);

    assert_eq!(
        super::parse(body.as_bytes()).expect_err("key length is bounded"),
        super::invalid()
    );
}
