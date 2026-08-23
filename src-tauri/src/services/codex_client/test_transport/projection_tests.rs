use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::super::sensitive_buffer::SensitiveBuffer;

#[test]
fn projection_has_one_raw_json_authority_without_serde_scratch() {
    for source in [
        include_str!("projection.rs"),
        include_str!("projection_lex.rs"),
        include_str!("projection_scan.rs"),
        include_str!("projection_state.rs"),
        include_str!("projection_string.rs"),
    ] {
        assert!(!source.contains("serde_json::from_slice"));
        assert!(!source.contains("serde_json::Deserializer"));
    }
}

#[test]
fn escaped_nested_sensitive_key_is_detected() {
    let body = br#"{
        "model":"gpt-5.6-sol",
        "input":[{"metadata":{"access\u005ftoken":"secret"}}],
        "tools":[]
    }"#;

    let projection = super::parse(body).expect("escaped key is valid JSON");

    assert!(projection.forbidden_field_present);
}

#[test]
fn escaped_root_fields_are_projected_and_arrays_counted_with_whitespace() {
    let body = br#"{
        "m\u006fdel" : "gpt-5.6-sol",
        "service\u005ftier" : "pri\u006frity",
        "type" : "response.create",
        "input" : [null, {"nested":"ignored"}],
        "tools" : [true]
    }"#;

    let projection = super::parse(body).expect("valid spaced JSON is projected");

    assert_eq!(projection.model, "gpt-5.6-sol");
    assert_eq!(projection.service_tier.as_deref(), Some("priority"));
    assert_eq!(projection.envelope_type.as_deref(), Some("response.create"));
    assert_eq!(projection.input_count, 2);
    assert_eq!(projection.tool_count, 1);
}

#[test]
fn escaped_key_longer_than_256_decoded_bytes_fails_closed() {
    let key = "\\u006b".repeat(257);
    let body = format!(r#"{{"model":"gpt-5.6-sol","input":[{{"{key}":null}}],"tools":[]}}"#);
    let key_scratch_zeroized = Arc::new(AtomicBool::new(false));

    let result = super::scan::parse_with_key_zeroize_hook(
        body.as_bytes(),
        Arc::clone(&key_scratch_zeroized),
    );

    assert!(result.is_err(), "decoded key length is bounded");
    assert!(key_scratch_zeroized.load(Ordering::SeqCst));
    assert_eq!(
        super::parse(body.as_bytes()).expect_err("public projection fails closed"),
        super::invalid()
    );
}

#[test]
fn escaped_secret_value_is_never_projected_and_raw_buffer_is_zeroized() {
    const SECRET_MARKER: &str = "secret-escaped-value-must-not-survive";
    let escaped_marker = SECRET_MARKER
        .bytes()
        .map(|byte| format!("\\u{byte:04x}"))
        .collect::<String>();
    let body = format!(
        r#"{{"model":"gpt-5.6-sol","input":[{{"content":"{escaped_marker}"}}],"tools":[]}}"#
    );
    let zeroized = Arc::new(AtomicBool::new(false));
    let mut buffer = SensitiveBuffer::with_capacity(body.len(), Arc::clone(&zeroized));
    buffer.extend_from_slice(body.as_bytes());

    let projection = super::parse(&buffer).expect("escaped secret value is valid JSON");

    assert!(!format!("{projection:?}").contains(SECRET_MARKER));
    assert!(!buffer
        .windows(SECRET_MARKER.len())
        .any(|window| window == SECRET_MARKER.as_bytes()));
    drop(buffer);
    assert!(zeroized.load(Ordering::SeqCst));
}

#[test]
fn malformed_string_escapes_and_surrogates_fail_closed() {
    for invalid_value in [
        r#""\q""#,
        r#""\uD800""#,
        r#""\uDC00""#,
        r#""\uD800\u0041""#,
        r#""\uZZZZ""#,
    ] {
        let body = format!(
            r#"{{"model":"gpt-5.6-sol","input":[],"tools":[],"metadata":{invalid_value}}}"#
        );

        assert_eq!(
            super::parse(body.as_bytes()).expect_err("malformed escape must be rejected"),
            super::invalid(),
            "value: {invalid_value}"
        );
    }
}

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
