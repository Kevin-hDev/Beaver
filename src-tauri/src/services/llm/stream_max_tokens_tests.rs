use super::choose;

#[test]
fn model_limit_replaces_the_provider_fallback() {
    assert_eq!(
        choose(None, Some(100_000), true, Some(128_000)),
        Some(100_000)
    );
    assert_eq!(
        choose(None, Some(16_384), true, Some(128_000)),
        Some(16_384)
    );
}

#[test]
fn explicit_limit_is_clamped_to_the_model_limit() {
    assert_eq!(
        choose(Some(128_000), Some(100_000), true, Some(128_000)),
        Some(100_000)
    );
    assert_eq!(
        choose(Some(32_000), Some(100_000), true, Some(128_000)),
        Some(32_000)
    );
}

#[test]
fn unknown_model_uses_the_safe_available_fallback() {
    assert_eq!(choose(None, None, true, Some(128_000)), Some(128_000));
    assert_eq!(choose(None, None, true, None), None);
    assert_eq!(choose(Some(8_000), None, true, Some(128_000)), Some(8_000));
}

#[test]
fn groq_and_cerebras_omit_automatic_limits() {
    assert_eq!(choose(None, Some(32_768), false, None), None);
    assert_eq!(choose(None, Some(128_000), false, None), None);
}

#[test]
fn explicit_limits_remain_available_when_automatic_limits_are_disabled() {
    assert_eq!(choose(Some(4_000), Some(32_768), false, None), Some(4_000));
    assert_eq!(
        choose(Some(64_000), Some(32_768), false, None),
        Some(32_768)
    );
}
