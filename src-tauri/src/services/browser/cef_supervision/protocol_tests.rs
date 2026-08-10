use super::{
    CefLaunchMarker, CefMarkerError, CefProcessRole, CefUnavailableCategory, CEF_SLOT_CAPACITY,
};

#[test]
fn only_the_reserved_cef_helper_role_is_accepted() {
    assert_eq!(CefProcessRole::try_from(1), Ok(CefProcessRole::Helper));
    assert!(CefProcessRole::try_from(0).is_err());
    assert!(CefProcessRole::try_from(2).is_err());
}

#[test]
fn a_generated_marker_round_trips_without_exposing_its_nonce() {
    let marker = CefLaunchMarker::generate(7, 9, CefProcessRole::Helper).expect("marker");
    let encoded = marker.encode();
    let decoded = CefLaunchMarker::decode_unique(&[encoded.as_str()]).expect("decode");

    assert_eq!(decoded, marker);
    assert_eq!(format!("{marker:?}"), "CefLaunchMarker([redacted])");
    assert!(!format!("{marker:?}").contains(encoded.as_str()));
}

#[test]
fn generated_markers_do_not_reuse_the_same_capability() {
    let first = CefLaunchMarker::generate(0, 1, CefProcessRole::Helper).expect("first");
    let second = CefLaunchMarker::generate(0, 1, CefProcessRole::Helper).expect("second");

    assert_ne!(first, second);
}

#[test]
fn missing_duplicate_or_malformed_markers_fail_closed() {
    assert_eq!(
        CefLaunchMarker::decode_unique(&[]),
        Err(CefMarkerError::Missing)
    );
    assert_eq!(
        CefLaunchMarker::decode_unique(&["v1:0:1:00", "v1:0:1:00"]),
        Err(CefMarkerError::Duplicate)
    );
    assert_eq!(
        CefLaunchMarker::decode_unique(&["v1:0:1:00"]),
        Err(CefMarkerError::Invalid)
    );
    assert_eq!(
        CefLaunchMarker::decode_unique(&[&"x".repeat(512)]),
        Err(CefMarkerError::Invalid)
    );
}

#[test]
fn slot_and_generation_boundaries_are_rejected() {
    assert!(CefLaunchMarker::generate(CEF_SLOT_CAPACITY, 1, CefProcessRole::Helper).is_err());
    assert!(CefLaunchMarker::generate(0, 0, CefProcessRole::Helper).is_err());
}

#[test]
fn unavailable_diagnostics_are_bounded_categories_only() {
    let codes = CefUnavailableCategory::ALL.map(CefUnavailableCategory::code);

    assert_eq!(
        codes,
        [
            "cef-supervision-object",
            "cef-supervision-permission",
            "cef-supervision-admission",
            "cef-supervision-reaper",
            "cef-supervision-sandbox",
        ]
    );
    assert!(codes.iter().all(|code| code.len() <= 32));
}
