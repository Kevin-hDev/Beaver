use crate::services::git::blob_preview;
use std::path::Path;

pub(super) async fn load_blob_with_limit(
    path: String,
    expected_branch: String,
    commit_id: String,
    file_path: String,
    use_parent: bool,
    max_size: usize,
) -> Result<Vec<u8>, String> {
    let repo_path = super::git::registered_project_path(&path).await?;
    tokio::task::spawn_blocking(move || {
        blob_preview::read_blob_with_limit(
            &repo_path,
            &expected_branch,
            &commit_id,
            &file_path,
            use_parent,
            max_size,
        )
    })
    .await
    .map_err(|_| unavailable())?
    .map_err(|_| unavailable())
}

pub(super) fn validate_extension(file_path: &str, allowed: &[&str]) -> Result<String, String> {
    let extension = Path::new(file_path)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(unavailable)?;
    allowed
        .contains(&extension.as_str())
        .then_some(())
        .ok_or_else(unavailable)?;
    Ok(extension)
}

fn unavailable() -> String {
    "Aperçu Git indisponible".to_string()
}
