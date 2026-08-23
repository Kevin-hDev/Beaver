#[test]
fn valid_surrogate_pair_in_ignored_value_is_accepted_without_projection() {
    let body = br#"{
        "model":"gpt-5.6-sol",
        "input":[{"content":"\uD83D\uDE00"}],
        "tools":[]
    }"#;

    let projection = super::parse(body).expect("valid surrogate pair is accepted");

    assert_eq!(projection.input_count, 1);
    assert!(!format!("{projection:?}").contains('😀'));
}

#[test]
fn malformed_json_shapes_fail_closed() {
    for body in [
        r#"["not-an-object"]"#,
        r#"{"model":"gpt-5.6-sol","input":[1,],"tools":[]}"#,
        r#"{"model":"gpt-5.6-sol","input":[],"tools":[],}"#,
        r#"{"model":"gpt-5.6-sol","input":[],"tools":[],"x":01}"#,
        r#"{"model":"gpt-5.6-sol","input":[],"tools":[],"x":1.}"#,
        r#"{"model":"gpt-5.6-sol","input":[],"tools":[],"x":true false}"#,
        r#"{"model":"gpt-5.6-sol","input":[],"tools":[]} trailing"#,
        "{\"model\":\"gpt-5.6-sol\",\"input\":[],\"tools\":[],\"x\":\"line\nfeed\"}",
    ] {
        assert_eq!(
            super::parse(body.as_bytes()).expect_err("malformed JSON must be rejected"),
            super::invalid(),
            "body: {body:?}"
        );
    }
}
