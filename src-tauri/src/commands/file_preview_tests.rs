use super::*;

#[tokio::test]
async fn check_preview_files_exist_reports_existing_and_missing_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("kept.txt"), "ok").expect("write file");

    let results = check_preview_files_exist(
        vec!["kept.txt".to_string(), "deleted.txt".to_string()],
        Some(dir.path().to_string_lossy().to_string()),
    )
    .await
    .expect("check files");

    assert_eq!(results.len(), 2);
    assert!(results[0].exists);
    assert!(!results[1].exists);
    assert_eq!(results[0].path, "kept.txt");
}

#[tokio::test]
async fn check_preview_files_exist_treats_invalid_paths_as_missing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let results = check_preview_files_exist(
        vec!["../secret.txt".to_string()],
        Some(dir.path().to_string_lossy().to_string()),
    )
    .await
    .expect("check files");

    assert_eq!(results.len(), 1);
    assert!(!results[0].exists);
}

#[test]
fn frontend_base_directory_does_not_become_an_authorized_root() {
    let allowed = tempfile::tempdir().expect("allowed root");
    let attacker_controlled = tempfile::tempdir().expect("untrusted root");
    let secret = attacker_controlled.path().join("secret.txt");
    std::fs::write(&secret, "secret").expect("write secret");

    let result = resolve_preview_path_with_roots(
        secret.to_str().expect("secret path"),
        attacker_controlled.path().to_str(),
        &[allowed.path().to_path_buf()],
    );

    assert!(result.is_err());
}
