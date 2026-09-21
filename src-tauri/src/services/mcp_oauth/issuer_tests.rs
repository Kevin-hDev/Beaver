use super::{verify_callback_issuer, verify_metadata_issuer};

#[test]
fn issuer_mismatch_is_rejected_even_with_a_valid_state() {
    let expected = "https://auth.example.test/tenant-a";
    assert!(verify_callback_issuer(expected, Some(expected), true).is_ok());
    assert!(verify_callback_issuer(expected, None, true).is_err());
    assert!(verify_callback_issuer(expected, None, false).is_ok());
    assert!(
        verify_callback_issuer(expected, Some("https://auth.example.test/tenant-b"), false)
            .is_err()
    );
}

#[test]
fn metadata_cannot_change_a_discovered_tenant() {
    assert!(verify_metadata_issuer(
        "https://auth.example.test/tenant-a",
        "https://auth.example.test/tenant-b",
    )
    .is_err());
}
