use crate::services::agent_local::security::validate_read_path;
use crate::services::agent_local::types_tools::ToolResult;
use std::path::Path;

const SUPPORTED_FORMATS: &[&str] = &["jpeg", "jpg", "png", "webp", "gif", "bmp"];

pub async fn read_image(path: &str, working_dir: &Path) -> ToolResult {
    if path.is_empty() {
        return ToolResult::validation("image_path_required", "Le paramètre 'path' est requis");
    }

    let resolved = super::tool_office_utils::resolve_path(path, working_dir);

    let validated = match validate_read_path(&resolved, working_dir) {
        Ok(p) => p,
        Err(error) => {
            return super::tool_file_error::path_failure(
                error,
                "image_not_found",
                "image_read_denied",
                "invalid_image_path",
            )
        }
    };

    let ext = validated
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    if !SUPPORTED_FORMATS.contains(&ext.as_str()) {
        return ToolResult::validation(
            "image_format_unsupported",
            "Format non supporté. Formats acceptés : jpeg, jpg, png, webp, gif, bmp",
        );
    }

    let file_size = match std::fs::metadata(&validated) {
        Ok(m) => m.len(),
        Err(error) => return super::tool_file_error::io_failure(error, "image_metadata_failed"),
    };

    let (width, height) = match image::image_dimensions(&validated) {
        Ok(dims) => dims,
        Err(_) => return ToolResult::validation(
            "image_dimensions_unreadable",
            "Impossible de lire les dimensions de l'image",
        ),
    };

    let format = normalize_format(&ext);
    let json = serde_json::json!({
        "width": width,
        "height": height,
        "format": format,
        "file_size_bytes": file_size
    });
    ToolResult::ok(json.to_string())
}

fn normalize_format(ext: &str) -> &str {
    match ext {
        "jpg" => "jpeg",
        other => other,
    }
}
