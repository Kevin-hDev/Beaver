use super::error::OllamaErrorCode;
use super::update::{platform, UpdateRequest};
use std::ffi::OsString;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[tokio::test]
async fn model_store_conflict_is_rejected_before_any_update_staging_mutation() {
    let root = tempfile::tempdir_in(".").expect("data root");
    let mut request = UpdateRequest::for_test(root.path().to_path_buf());
    let bin = request.paths.active.join("bin");
    std::fs::create_dir_all(&bin).expect("active bin");
    let executable = bin.join(if cfg!(windows) {
        "ollama.exe"
    } else {
        "ollama"
    });
    std::fs::write(&executable, b"active").expect("active executable");
    #[cfg(unix)]
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755))
        .expect("active permissions");
    request.inherited_environment = vec![(
        OsString::from("OLLAMA_MODELS"),
        request.paths.update_staging.as_os_str().to_owned(),
    )];

    assert_eq!(
        platform::run(request.clone()).await,
        Err(OllamaErrorCode::OllamaModelStoreConflict)
    );
    assert!(!request.paths.update_staging.exists());
    assert!(!request.paths.archive_staging.exists());
    assert!(!request.paths.journal.exists());
}
