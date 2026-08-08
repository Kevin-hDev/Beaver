use super::ollama_parameter_summary::parse;

#[test]
fn decodes_ollama_go_escaped_string_values() {
    let summary = concat!(
        "stop                           \"\\\"User:\\\"\"\n",
        "stop                           \" line one\\nline two \"\n",
        "future_option                  \"C:\\\\models\\tvalue\"",
    );

    assert_eq!(
        parse(summary).expect("valid Ollama parameter summary"),
        vec![
            ("stop".into(), "\"User:\"".into()),
            ("stop".into(), " line one\nline two ".into()),
            ("future_option".into(), "C:\\models\tvalue".into()),
        ]
    );
}

#[test]
fn preserves_unquoted_numeric_and_boolean_values() {
    let summary = concat!(
        "temperature                    0.7\n",
        "num_ctx                        24576\n",
        "future_flag                    true",
    );

    assert_eq!(
        parse(summary).expect("valid scalar values"),
        vec![
            ("temperature".into(), "0.7".into()),
            ("num_ctx".into(), "24576".into()),
            ("future_flag".into(), "true".into()),
        ]
    );
}

#[test]
fn supports_go_hex_unicode_and_control_escapes() {
    let summary = "stop                           \"\\x41\\u00e9\\U0001f44b\\a\\b\\f\\v\"";

    assert_eq!(
        parse(summary).expect("valid Go escapes"),
        vec![("stop".into(), "Aé👋\u{7}\u{8}\u{c}\u{b}".into())]
    );
}

#[test]
fn rejects_malformed_or_unbounded_summaries() {
    assert!(parse("stop").is_err());
    assert!(parse("stop                           \"unterminated").is_err());
    assert!(parse("stop                           \"bad\\q\"").is_err());

    let too_many = (0..129)
        .map(|_| "stop                           \"x\"")
        .collect::<Vec<_>>()
        .join("\n");
    assert!(parse(&too_many).is_err());

    let too_long = format!("stop                           \"{}\"", "x".repeat(1025));
    assert!(parse(&too_long).is_err());
}
