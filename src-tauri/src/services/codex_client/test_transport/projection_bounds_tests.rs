#[test]
fn input_at_2048_items_is_accepted() {
    let input = vec!["null"; 2_048].join(",");
    let body = format!(r#"{{"model":"gpt-5.6-sol","input":[{input}],"tools":[]}}"#);

    let projection = super::parse(body.as_bytes()).expect("exact item limit is accepted");

    assert_eq!(projection.input_count, 2_048);
}

#[test]
fn input_over_2048_items_fails_closed() {
    let input = vec!["null"; 2_049].join(",");
    let body = format!(r#"{{"model":"gpt-5.6-sol","input":[{input}],"tools":[]}}"#);

    assert_eq!(
        super::parse(body.as_bytes()).expect_err("item count is bounded"),
        super::invalid()
    );
}
