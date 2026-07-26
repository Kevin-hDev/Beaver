use crate::commands::ollama_bundle_utils::is_valid_semver;
use crate::commands::ollama_setup::fallback_ollama_version;
use tokio_util::sync::CancellationToken;

#[test]
fn valid_semver_accepted() {
    assert!(is_valid_semver("0.23.1"));
    assert!(is_valid_semver("1.0.0"));
    assert!(is_valid_semver("0.30.0-rc3"));
    assert!(is_valid_semver("2.1.0-beta.1"));
}

#[test]
fn invalid_semver_rejected() {
    assert!(!is_valid_semver(""));
    assert!(!is_valid_semver("1.0"));
    assert!(!is_valid_semver("abc"));
    assert!(!is_valid_semver("1.0.0/../../evil"));
    assert!(!is_valid_semver("1.0.0%0d%0aHeader: inject"));
    assert!(!is_valid_semver("1.0.0\nmalicious"));
    assert!(!is_valid_semver("v1.0.0"));
}

#[test]
fn fallback_install_version_is_current_supported_release() {
    assert_eq!(fallback_ollama_version(), "0.32.1");
    assert!(is_valid_semver(fallback_ollama_version()));
}

#[test]
fn cancelled_error_is_detected_exactly() {
    let err = crate::commands::ollama_setup_cancel::cancelled_error();

    assert!(crate::commands::ollama_setup_cancel::is_cancelled_error(
        &err
    ));
    assert!(!crate::commands::ollama_setup_cancel::is_cancelled_error(
        "cancelled"
    ));
}

#[tokio::test]
async fn cancel_active_setup_cancels_registered_token() {
    let token = CancellationToken::new();
    crate::commands::ollama_setup_cancel::register(token.clone()).await;

    crate::commands::ollama_setup_cancel::cancel_active().await;

    assert!(token.is_cancelled());
    crate::commands::ollama_setup_cancel::clear().await;
}
