use super::choose;

#[test]
fn model_limit_replaces_the_provider_fallback() {
    assert_eq!(choose(None, Some(100_000), Some(128_000)), Some(100_000));
    assert_eq!(choose(None, Some(16_384), Some(128_000)), Some(16_384));
}

#[test]
fn explicit_limit_is_clamped_to_the_model_limit() {
    assert_eq!(
        choose(Some(128_000), Some(100_000), Some(128_000)),
        Some(100_000)
    );
    assert_eq!(
        choose(Some(32_000), Some(100_000), Some(128_000)),
        Some(32_000)
    );
}

#[test]
fn unknown_model_uses_the_safe_available_fallback() {
    assert_eq!(choose(None, None, Some(128_000)), Some(128_000));
    assert_eq!(choose(None, None, None), None);
    assert_eq!(choose(Some(8_000), None, Some(128_000)), Some(8_000));
}
