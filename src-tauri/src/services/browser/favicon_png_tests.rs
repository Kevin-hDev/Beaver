use super::{favicon_png::encode, favicon_policy::MAX_PNG_BYTES};

#[test]
fn invalid_dimensions_and_size_are_rejected_before_copy() {
    for (w, h, size) in [
        (0, 16, 8),
        (16, 65, 8),
        (16, 16, 0),
        (16, 16, MAX_PNG_BYTES + 1),
    ] {
        assert!(encode(w, h, size, |_| panic!("must not access rejected binary")).is_none());
    }
}

#[test]
fn partial_copy_and_wrong_signature_are_rejected() {
    assert!(encode(16, 16, 8, |_| 7).is_none());
    assert!(encode(16, 16, 8, |_| 8).is_none());
    assert_eq!(
        encode(16, 16, 8, |bytes| {
            bytes.copy_from_slice(b"\x89PNG\r\n\x1a\n");
            8
        })
        .as_deref(),
        Some("iVBORw0KGgo=")
    );
}
