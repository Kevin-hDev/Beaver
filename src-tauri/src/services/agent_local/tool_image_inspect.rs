use super::types_tools::ToolResult;
use image::DynamicImage;
use std::path::Path;

pub(super) fn inspect(path: &Path, image: &DynamicImage) -> ToolResult {
    let file_size = match std::fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(_) => {
            return ToolResult::execution(
                "image_metadata_failed",
                "Impossible de lire les métadonnées de l'image",
                true,
            )
        }
    };
    let format = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .map(|extension| {
            if extension == "jpg" {
                "jpeg".to_string()
            } else {
                extension
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    ToolResult::ok(
        serde_json::json!({
            "width": image.width(),
            "height": image.height(),
            "format": format,
            "file_size_bytes": file_size
        })
        .to_string(),
    )
}
