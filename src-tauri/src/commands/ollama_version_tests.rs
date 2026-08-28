use super::binary_update_from_versions;
use crate::services::ollama_manager::{OllamaErrorCode, OllamaVersion};

#[test]
fn failed_remote_lookup_is_not_reported_as_no_update() {
    let result =
        binary_update_from_versions("0.32.15".into(), Err(OllamaErrorCode::OllamaDownloadFailed));

    assert_eq!(result.unwrap_err(), "ollama-update-check-failed");
}

#[test]
fn successful_remote_lookup_distinguishes_current_and_newer_versions() {
    let current =
        binary_update_from_versions("0.33.1".into(), Ok(OllamaVersion::parse("0.33.1").unwrap()))
            .unwrap();
    assert!(current.is_none());

    let available = binary_update_from_versions(
        "0.32.15".into(),
        Ok(OllamaVersion::parse("0.33.1").unwrap()),
    )
    .unwrap()
    .expect("newer version");
    assert_eq!(available.current_version, "0.32.15");
    assert_eq!(available.latest_version, "0.33.1");
}
