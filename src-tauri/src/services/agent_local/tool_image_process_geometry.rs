use image::imageops::FilterType;
use image::{DynamicImage, ImageReader};
use serde_json::Value;
use std::path::Path;

use super::types_tools::ToolResult;

pub(super) const MAX_DIMENSION: u32 = 8000;
pub(super) const MIN_QUALITY: u64 = 1;
pub(super) const MAX_QUALITY: u64 = 100;
const MAX_PIXELS: u64 = 50_000_000;

pub(super) fn resize(img: DynamicImage, op: &Value) -> Result<DynamicImage, ToolResult> {
    let width = bounded_dimension(&op["width"], "resize: 'width' requis")?;
    let height = bounded_dimension(&op["height"], "resize: 'height' requis")?;
    ensure_pixel_budget(width, height)?;
    let mode = match op.get("mode") {
        None | Some(Value::Null) => "fit",
        Some(Value::String(mode)) if matches!(mode.as_str(), "fit" | "fill" | "exact") => mode,
        Some(_) => {
            return Err(ToolResult::validation(
                "image_resize_mode_invalid",
                "resize: 'mode' doit être fit, fill ou exact",
            ))
        }
    };
    let resized = match mode {
        "fill" => img.resize_to_fill(width, height, FilterType::Lanczos3),
        "exact" => img.resize_exact(width, height, FilterType::Lanczos3),
        _ => img.resize(width, height, FilterType::Lanczos3),
    };
    Ok(resized)
}

pub(super) fn crop(img: DynamicImage, op: &Value) -> Result<DynamicImage, ToolResult> {
    let x = bounded_coordinate(&op["x"], "crop: 'x' requis")?;
    let y = bounded_coordinate(&op["y"], "crop: 'y' requis")?;
    let width = bounded_dimension(&op["width"], "crop: 'width' requis")?;
    let height = bounded_dimension(&op["height"], "crop: 'height' requis")?;
    ensure_pixel_budget(width, height)?;
    if x.saturating_add(width) > img.width() || y.saturating_add(height) > img.height() {
        return Err(ToolResult::validation(
            "image_crop_out_of_bounds",
            "crop hors limites de l'image",
        ));
    }
    Ok(img.crop_imm(x, y, width, height))
}

pub(super) fn quality(op: &Value) -> Result<u8, ToolResult> {
    let value = op["value"].as_u64().ok_or_else(|| {
        ToolResult::validation("image_quality_required", "quality: 'value' requis")
    })?;
    if !(MIN_QUALITY..=MAX_QUALITY).contains(&value) {
        return Err(ToolResult::validation(
            "image_quality_out_of_range",
            "quality: 'value' doit être compris entre 1 et 100",
        ));
    }
    u8::try_from(value).map_err(|_| {
        ToolResult::validation(
            "image_quality_out_of_range",
            "quality: 'value' doit être compris entre 1 et 100",
        )
    })
}

pub(super) fn validate_output_format(path: &Path) -> Result<(), ToolResult> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);
    if extension.as_deref().is_some_and(|value| {
        matches!(value, "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp")
    }) {
        return Ok(());
    }
    Err(ToolResult::validation(
        "image_output_format_unsupported",
        "Format de sortie non supporté : jpg, jpeg, png, webp, gif ou bmp requis",
    ))
}

pub(super) fn validate_dimensions(path: &Path) -> Result<(), String> {
    let reader = ImageReader::open(path)
        .map_err(|_| "Impossible d'ouvrir l'image".to_string())?
        .with_guessed_format()
        .map_err(|_| "Format image invalide".to_string())?;
    let (width, height) = reader
        .into_dimensions()
        .map_err(|_| "Impossible de lire les dimensions de l'image".to_string())?;
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err("Dimensions image non supportées".to_string());
    }
    ensure_pixel_budget(width, height).map_err(|error| error.content)
}

fn bounded_dimension(value: &Value, missing: &str) -> Result<u32, ToolResult> {
    let raw = value
        .as_u64()
        .ok_or_else(|| ToolResult::validation("image_dimension_required", missing))?;
    let dimension = u32::try_from(raw).map_err(|_| {
        ToolResult::validation("image_dimension_too_large", "Dimension trop grande")
    })?;
    if dimension == 0 || dimension > MAX_DIMENSION {
        return Err(ToolResult::validation(
            "image_dimension_out_of_range",
            "Dimension hors limites",
        ));
    }
    Ok(dimension)
}

fn bounded_coordinate(value: &Value, missing: &str) -> Result<u32, ToolResult> {
    let raw = value
        .as_u64()
        .ok_or_else(|| ToolResult::validation("image_coordinate_required", missing))?;
    u32::try_from(raw).map_err(|_| {
        ToolResult::validation("image_coordinate_too_large", "Coordonnée trop grande")
    })
}

fn ensure_pixel_budget(width: u32, height: u32) -> Result<(), ToolResult> {
    let pixels = u64::from(width).saturating_mul(u64::from(height));
    if pixels > MAX_PIXELS {
        return Err(ToolResult::validation(
            "image_pixel_limit_exceeded",
            "Image trop grande",
        ));
    }
    Ok(())
}
