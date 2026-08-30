use super::summary_contract::{validate, SummaryRawOutput};

pub(super) fn valid_output() -> String {
    let body = super::summary_contract::required_sections()
        .into_iter()
        .map(|section| format!("{section}\nDetails."))
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("<summary>\n{body}\n</summary>")
}

fn output(content: String) -> SummaryRawOutput {
    SummaryRawOutput {
        content,
        tool_call_count: 0,
        truncated: false,
        cancelled: false,
    }
}

#[test]
fn accepts_one_complete_bounded_envelope() {
    let validated = validate(output(valid_output()), 2_000).unwrap();
    assert!(validated.content.contains("9. Next Step"));
    assert!(validated.estimated_tokens > 0);
}

#[test]
fn rejects_missing_empty_duplicate_or_trailing_envelopes() {
    for content in [
        "plain text".to_string(),
        "<summary> </summary>".to_string(),
        format!("{}\n{}", valid_output(), valid_output()),
        format!("{} trailing", valid_output()),
        "<summary>1. Primary Request and Intent</summary>".to_string(),
    ] {
        assert!(validate(output(content), 2_000).is_err());
    }
}

#[test]
fn rejects_truncation_tool_calls_cancellation_and_budget_overflow() {
    let mut raw = output(valid_output());
    raw.truncated = true;
    assert!(validate(raw, 2_000).is_err());

    let mut raw = output(valid_output());
    raw.tool_call_count = 1;
    assert!(validate(raw, 2_000).is_err());

    let mut raw = output(valid_output());
    raw.cancelled = true;
    assert!(validate(raw, 2_000).is_err());

    assert!(validate(output(valid_output()), 1).is_err());
}
