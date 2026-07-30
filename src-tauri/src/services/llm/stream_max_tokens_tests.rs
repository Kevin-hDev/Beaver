use super::{choose, select_sources, ResolveError};
use crate::services::llm::provider_model_lookup::ModelLimits;

fn resolved(
    requested: Option<u32>,
    model_limit: Option<u32>,
    auto_max_tokens: bool,
    provider_fallback: Option<u32>,
) -> Option<u32> {
    choose(
        requested,
        model_limit,
        None,
        auto_max_tokens,
        provider_fallback,
        None,
        0,
    )
    .unwrap()
}

#[test]
fn model_limit_replaces_the_provider_fallback() {
    assert_eq!(
        resolved(None, Some(100_000), true, Some(128_000)),
        Some(100_000)
    );
    assert_eq!(
        resolved(None, Some(16_384), true, Some(128_000)),
        Some(16_384)
    );
}

#[test]
fn local_registry_is_authoritative_over_runtime_and_fallback_data() {
    let local = ModelLimits {
        context_window: Some(262_144),
        max_output_tokens: None,
        default_output_tokens: Some(32_000),
    };
    let fallback = ModelLimits {
        context_window: Some(8_192),
        max_output_tokens: Some(8_192),
        default_output_tokens: None,
    };

    assert_eq!(
        select_sources(
            Some(local),
            Some((Some(16_384), Some(4_096))),
            Some(fallback)
        ),
        (Some(262_144), None, Some(32_000))
    );
    assert_eq!(
        select_sources(None, Some((Some(16_384), None)), Some(fallback)),
        (Some(16_384), Some(8_192), None)
    );
}

#[test]
fn documented_model_default_is_distinct_from_its_maximum() {
    assert_eq!(
        choose(
            None,
            Some(1_048_576),
            Some(131_072),
            true,
            Some(64_000),
            Some(1_048_576),
            1_000,
        ),
        Ok(Some(131_072))
    );
    assert_eq!(
        choose(
            Some(500_000),
            Some(1_048_576),
            Some(131_072),
            true,
            Some(64_000),
            Some(1_048_576),
            1_000,
        ),
        Ok(Some(500_000))
    );
}

#[test]
fn explicit_limit_is_clamped_to_the_model_limit() {
    assert_eq!(
        resolved(Some(128_000), Some(100_000), true, Some(128_000)),
        Some(100_000)
    );
    assert_eq!(
        resolved(Some(32_000), Some(100_000), true, Some(128_000)),
        Some(32_000)
    );
}

#[test]
fn unknown_model_uses_the_safe_available_fallback() {
    assert_eq!(resolved(None, None, true, Some(128_000)), Some(128_000));
    assert_eq!(resolved(None, None, true, None), None);
    assert_eq!(
        resolved(Some(8_000), None, true, Some(128_000)),
        Some(8_000)
    );
}

#[test]
fn groq_and_cerebras_omit_automatic_limits() {
    assert_eq!(resolved(None, Some(32_768), false, None), None);
    assert_eq!(resolved(None, Some(128_000), false, None), None);
}

#[test]
fn explicit_limits_remain_available_when_automatic_limits_are_disabled() {
    assert_eq!(
        resolved(Some(4_000), Some(32_768), false, None),
        Some(4_000)
    );
    assert_eq!(
        resolved(Some(64_000), Some(32_768), false, None),
        Some(32_768)
    );
}

#[test]
fn automatic_limit_never_exceeds_the_remaining_context() {
    assert_eq!(
        choose(None, None, None, true, Some(64_000), Some(40_000), 34_000),
        Ok(Some(6_000))
    );
    assert_eq!(
        choose(None, None, None, true, Some(131_072), Some(8_192), 6_000),
        Ok(Some(2_192))
    );
    assert_eq!(
        choose(
            None,
            None,
            None,
            true,
            Some(64_000),
            Some(1_000_000),
            950_000
        ),
        Ok(Some(50_000))
    );
}

#[test]
fn explicit_limit_is_also_clamped_to_the_remaining_context() {
    assert_eq!(
        choose(
            Some(100_000),
            Some(128_000),
            None,
            true,
            Some(128_000),
            Some(200_000),
            175_000,
        ),
        Ok(Some(25_000))
    );
}

#[test]
fn exhausted_context_and_zero_requests_fail_closed() {
    assert_eq!(
        choose(None, None, None, false, None, Some(8_192), 8_192),
        Err(ResolveError::ContextExhausted)
    );
    assert_eq!(
        choose(Some(0), None, None, true, None, Some(8_192), 1_000),
        Err(ResolveError::InvalidLimit)
    );
}
