pub(super) fn reconcile(
    _value: &serde_json::Value,
    context: super::UsageContext<'_>,
    usage: &mut super::RequestUsage,
) {
    if context.canonical_provider_id != "google"
        || context.api_format != super::UsageApiFormat::ChatCompletions
    {
        return;
    }
    #[cfg(debug_assertions)]
    if crate::services::reasoning_fixture_budget::is_active() {
        // Debug fixtures only: fixed numeric projection, never arbitrary keys/body.
        log::info!(
            "google_usage_received fields={} numbers={:?}",
            _value.as_object().map_or(0, |fields| fields.len()),
            numeric_evidence(_value)
        );
    }
    // Google compatibility live-verified 2026-09-08: completion_tokens can
    // exclude thoughts. Google's total is prompt + thoughts + candidates:
    // https://ai.google.dev/api/generate-content#UsageMetadata
    // Reconcile generated output once for both stream readers and pricing.
    // Do not invent reasoning/cache detail, or price contradictory totals.
    let generated = usage
        .input_tokens
        .zip(usage.total_tokens)
        .and_then(|(input, total)| total.checked_sub(input))
        .filter(|generated| {
            usage
                .output_tokens
                .is_none_or(|output| output <= *generated)
        })
        .filter(|generated| {
            usage
                .reasoning_output_tokens
                .is_none_or(|reasoning| reasoning <= *generated)
        });
    usage.output_tokens = generated;
    if generated.is_none() {
        usage.reasoning_output_tokens = None;
    }
}

#[cfg(any(debug_assertions, test))]
fn numeric_evidence(value: &serde_json::Value) -> [Option<u64>; 5] {
    [
        "/prompt_tokens",
        "/completion_tokens",
        "/total_tokens",
        "/completion_tokens_details/reasoning_tokens",
        "/prompt_tokens_details/cached_tokens",
    ]
    .map(|path| {
        value
            .pointer(path)
            .and_then(serde_json::Value::as_u64)
            .filter(|count| *count <= super::MAX_REQUEST_TOKENS)
    })
}

#[cfg(test)]
mod tests {
    use super::super::{RequestUsage, UsageApiFormat, UsageContext};

    #[test]
    fn google_chat_total_includes_unitemized_generated_tokens() {
        for (input, completion, total, expected) in
            [(6134, 1, 6165, 31), (6180, 1, 6206, 26), (134, 3, 217, 83)]
        {
            let usage = RequestUsage::from_json_with_context(
                &serde_json::json!({"prompt_tokens":input,"completion_tokens":completion,"total_tokens":total}),
                UsageContext::chat("google", "gemini-3.8-flash"),
            ).unwrap();
            assert_eq!(usage.output_tokens, Some(expected));
            assert_eq!(usage.reasoning_output_tokens, None);
            assert_eq!(usage.cached_input_tokens, None);
            assert_eq!(usage.total_tokens, Some(total));
        }
    }

    #[test]
    fn google_chat_preserves_explicit_reasoning_without_counting_it_twice() {
        for completion in [18, 96] {
            let usage = RequestUsage::from_json_with_context(
                &serde_json::json!({"prompt_tokens":15,"completion_tokens":completion,
                    "total_tokens":111,"completion_tokens_details":{"reasoning_tokens":78}}),
                UsageContext::chat("google", "gemini-3.8-flash"),
            )
            .unwrap();
            assert_eq!(usage.output_tokens, Some(96));
            assert_eq!(usage.reasoning_output_tokens, Some(78));
        }
    }

    #[test]
    fn other_routes_keep_their_existing_usage_contract() {
        let value =
            serde_json::json!({"prompt_tokens":15,"completion_tokens":18,"total_tokens":111});
        for provider in [
            "openai",
            "openrouter",
            "zai",
            "anthropic",
            "qwen",
            "unknown",
        ] {
            let usage = RequestUsage::from_json_with_context(
                &value,
                UsageContext::chat(provider, "fixture"),
            )
            .unwrap();
            assert_eq!(usage.output_tokens, Some(18), "{provider}");
        }
        let usage = RequestUsage::from_json_with_context(
            &value,
            UsageContext {
                canonical_provider_id: "google",
                model: "fixture",
                api_format: UsageApiFormat::Responses,
            },
        )
        .unwrap();
        assert_eq!(usage.output_tokens, Some(18));
    }

    #[test]
    fn google_chat_does_not_price_missing_or_contradictory_totals() {
        for value in [
            serde_json::json!({"prompt_tokens":10,"completion_tokens":2}),
            serde_json::json!({"prompt_tokens":10,"completion_tokens":2,"total_tokens":null}),
            serde_json::json!({"prompt_tokens":10,"completion_tokens":2,"total_tokens":-1}),
            serde_json::json!({"prompt_tokens":10,"completion_tokens":2,"total_tokens":9}),
            serde_json::json!({"prompt_tokens":10,"completion_tokens":2,"total_tokens":11}),
            serde_json::json!({"prompt_tokens":10,"completion_tokens":2,"total_tokens":u64::MAX}),
            serde_json::json!({"prompt_tokens":10,"completion_tokens":2,"total_tokens":12,
                "completion_tokens_details":{"reasoning_tokens":3}}),
        ] {
            let usage = RequestUsage::from_json_with_context(
                &value,
                UsageContext::chat("google", "gemini-3.8-flash"),
            )
            .unwrap();
            assert_eq!(usage.output_tokens, None, "{value}");
            assert_eq!(usage.reasoning_output_tokens, None, "{value}");
        }
    }

    #[test]
    fn google_chat_keeps_zero_output_and_does_not_invent_cache_or_reasoning() {
        let usage = RequestUsage::from_json_with_context(
            &serde_json::json!({"prompt_tokens":10,"completion_tokens":0,"total_tokens":10}),
            UsageContext::chat("google", "gemini-3.8-flash"),
        )
        .unwrap();
        assert_eq!(usage.output_tokens, Some(0));
        assert_eq!(usage.reasoning_output_tokens, None);
        assert_eq!(usage.cached_input_tokens, None);
    }

    #[test]
    fn evidence_contains_only_bounded_numeric_usage_not_external_text() {
        let usage = serde_json::json!({
            "prompt_tokens": 6134, "completion_tokens": 1, "total_tokens": 6165,
            "completion_tokens_details": {"reasoning_tokens": "private-value"},
            "prompt_tokens_details": {"cached_tokens": u64::MAX},
            "private-field": "private-value"
        });
        let evidence = super::numeric_evidence(&usage);
        assert_eq!(evidence, [Some(6134), Some(1), Some(6165), None, None]);
        assert!(super::numeric_evidence(&serde_json::json!(null))
            .iter()
            .all(Option::is_none));
    }
}
