use super::delivery::Delivery;

#[test]
fn transcript_limit_counts_unicode_characters() {
    for accepted in [99_999, 100_000] {
        assert!(Delivery::new("op".into(), "draft".into(), "🦫".repeat(accepted), 0, 1, 1).is_ok());
    }
    assert!(Delivery::new("op".into(), "draft".into(), "🦫".repeat(100_001), 0, 1, 1).is_err());
}

#[test]
fn delivery_is_readable_until_its_exact_deadline() {
    let delivery =
        Delivery::new("op".into(), "draft".into(), "texte".into(), 120_000, 1, 1).unwrap();
    assert!(!delivery.is_expired_at(719_999));
    assert!(delivery.is_expired_at(720_000));
}
