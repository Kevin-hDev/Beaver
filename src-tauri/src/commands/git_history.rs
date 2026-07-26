use crate::services::git::{blob_preview, commit_files, diff_preview, history};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use std::io::Write;

const MAX_TEXT_SIZE: usize = 2 * 1024 * 1024;
const BINARY_EXTENSIONS: &[&str] = &["docx", "pdf"];
const SPREADSHEET_EXTENSIONS: &[&str] = &["xlsx", "xls", "csv", "ods", "xlsm", "tsv"];

#[tauri::command]
pub async fn list_git_commits(
    path: String,
    expected_branch: String,
    cursor: Option<String>,
    limit: Option<usize>,
) -> Result<history::CommitPage, String> {
    let repo_path = super::git::registered_project_path(&path).await?;
    tokio::task::spawn_blocking(move || {
        history::list_commits(&repo_path, &expected_branch, cursor.as_deref(), limit)
    })
    .await
    .map_err(|_| unavailable())?
    .map_err(|_| unavailable())
}

#[tauri::command]
pub async fn list_git_commit_files(
    path: String,
    expected_branch: String,
    commit_id: String,
) -> Result<Vec<commit_files::CommitFile>, String> {
    let repo_path = super::git::registered_project_path(&path).await?;
    tokio::task::spawn_blocking(move || {
        commit_files::list_commit_files(&repo_path, &expected_branch, &commit_id)
    })
    .await
    .map_err(|_| unavailable())?
    .map_err(|_| unavailable())
}

#[tauri::command]
pub async fn list_git_uncommitted_files(
    path: String,
    expected_branch: String,
) -> Result<history::UncommittedSnapshot, String> {
    let repo_path = super::git::registered_project_path(&path).await?;
    tokio::task::spawn_blocking(move || history::list_uncommitted(&repo_path, &expected_branch))
        .await
        .map_err(|_| unavailable())?
        .map_err(|_| unavailable())
}

#[tauri::command]
pub async fn read_git_diff_preview(
    path: String,
    expected_branch: String,
    commit_id: String,
    file_path: String,
    previous_path: Option<String>,
    mode: String,
) -> Result<diff_preview::GitDiffPreview, String> {
    let repo_path = super::git::registered_project_path(&path).await?;
    tokio::task::spawn_blocking(move || match mode.as_str() {
        "commit" => diff_preview::read_commit_diff(
            &repo_path,
            &expected_branch,
            &commit_id,
            &file_path,
            previous_path.as_deref(),
        ),
        "working" => diff_preview::read_working_diff(
            &repo_path,
            &expected_branch,
            &commit_id,
            &file_path,
            previous_path.as_deref(),
        ),
        _ => Err(unavailable()),
    })
    .await
    .map_err(|_| unavailable())?
    .map_err(|_| unavailable())
}

#[tauri::command]
pub async fn read_git_file_preview(
    path: String,
    expected_branch: String,
    commit_id: String,
    file_path: String,
    use_parent: bool,
) -> Result<String, String> {
    let bytes = super::git_history_preview::load_blob_with_limit(
        path,
        expected_branch,
        commit_id,
        file_path,
        use_parent,
        MAX_TEXT_SIZE,
    )
    .await?;
    if bytes.len() > MAX_TEXT_SIZE || bytes.contains(&0) {
        return Err(unavailable());
    }
    String::from_utf8(bytes).map_err(|_| unavailable())
}

#[tauri::command]
pub async fn read_git_binary_preview(
    path: String,
    expected_branch: String,
    commit_id: String,
    file_path: String,
    use_parent: bool,
) -> Result<String, String> {
    let _ = super::git_history_preview::validate_extension(&file_path, BINARY_EXTENSIONS)?;
    let bytes = super::git_history_preview::load_blob_with_limit(
        path,
        expected_branch,
        commit_id,
        file_path,
        use_parent,
        blob_preview::MAX_GIT_BINARY_PREVIEW_SIZE,
    )
    .await?;
    Ok(B64.encode(bytes))
}

#[tauri::command]
pub async fn read_git_spreadsheet_preview(
    path: String,
    expected_branch: String,
    commit_id: String,
    file_path: String,
    use_parent: bool,
    sheet: Option<String>,
    max_rows: Option<usize>,
) -> Result<String, String> {
    let extension =
        super::git_history_preview::validate_extension(&file_path, SPREADSHEET_EXTENSIONS)?;
    let bytes = super::git_history_preview::load_blob_with_limit(
        path,
        expected_branch,
        commit_id,
        file_path,
        use_parent,
        blob_preview::MAX_GIT_BLOB_SIZE,
    )
    .await?;
    let mut temp = tempfile::Builder::new()
        .suffix(&format!(".{extension}"))
        .tempfile()
        .map_err(|_| unavailable())?;
    temp.write_all(&bytes).map_err(|_| unavailable())?;
    super::file_preview_office::read_spreadsheet_path(temp.path().to_path_buf(), sheet, max_rows)
        .await
        .map_err(|_| unavailable())
}

fn unavailable() -> String {
    "Aperçu Git indisponible".to_string()
}
