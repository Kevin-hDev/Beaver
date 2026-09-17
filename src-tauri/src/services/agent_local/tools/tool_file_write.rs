use super::tool_file_error::io_failure;
use super::types_tools::ToolResult;
use crate::services::agent_local::security;
use crate::services::agent_local::tool_result_contract::ToolErrorCategory;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

pub async fn write_file(path: &str, content: &str, working_dir: &Path) -> ToolResult {
    let roots = security::allowed_write_roots_for(Some(working_dir));
    write_file_in_roots(path, content, working_dir, &roots).await
}

pub(crate) async fn write_file_in_roots(
    path: &str,
    content: &str,
    working_dir: &Path,
    roots: &[PathBuf],
) -> ToolResult {
    let raw = absolute_or_joined(path, working_dir);
    if matches!(
        tokio::fs::symlink_metadata(&raw).await,
        Ok(meta) if meta.file_type().is_symlink()
    ) {
        return symlink_error();
    }
    let resolved = match security::validate_write_path_in_roots(&raw, roots) {
        Ok(path) => path,
        Err(error) => {
            return super::tool_file_error::path_failure(
                error,
                "parent_directory_not_found",
                "write_path_denied",
                "invalid_write_path",
            )
        }
    };
    if let Some(parent) = resolved.parent() {
        if let Ok(real_parent) = dunce::canonicalize(parent) {
            if !roots.iter().any(|root| real_parent.starts_with(root)) {
                return ToolResult::error(
                    "Écriture interdite hors des zones autorisées",
                    "write_path_denied",
                    ToolErrorCategory::Permission,
                    false,
                );
            }
        }
        if let Err(error) = tokio::fs::create_dir_all(parent).await {
            return io_failure(error, "directory_create_failed");
        }
    }
    if resolved.is_symlink() {
        return symlink_error();
    }
    match write_no_follow(&resolved, content).await {
        Ok(()) => ToolResult::ok(format!("Écrit: {}", resolved.display())),
        Err(error) => io_failure(error, "file_write_failed"),
    }
}

fn absolute_or_joined(path: &str, working_dir: &Path) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        working_dir.join(path)
    }
}

fn symlink_error() -> ToolResult {
    ToolResult::error(
        "Écriture sur symlink interdite",
        "symlink_write_denied",
        ToolErrorCategory::Permission,
        false,
    )
}

async fn write_no_follow(path: &Path, content: &str) -> std::io::Result<()> {
    let metadata = tokio::fs::symlink_metadata(path).await;
    if matches!(metadata, Ok(meta) if meta.file_type().is_symlink()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "symlink",
        ));
    }
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW);
    let mut file = options.open(path).await?;
    file.write_all(content.as_bytes()).await?;
    file.flush().await
}
