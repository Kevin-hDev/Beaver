use crate::services::agent_local::security::{validate_read_path, validate_write_path};
use crate::services::agent_local::tool_office_limits::{
    ensure_source_size, MAX_IMAGE_SOURCE_BYTES,
};
use crate::services::agent_local::types_tools::ToolResult;
use image::imageops::FilterType;
use image::DynamicImage;
use image::ImageReader;
use serde_json::Value;
use std::path::Path;

const MAX_DIMENSION: u32 = 8000;
const MAX_PIXELS: u64 = 50_000_000;

pub async fn process_image(
    input_path: &str,
    output_path: &str,
    operations: &Value,
    working_dir: &Path,
) -> ToolResult {
    if input_path.is_empty() {
        return ToolResult::err("Le paramètre 'input_path' est requis");
    }
    if output_path.is_empty() {
        return ToolResult::err("Le paramètre 'output_path' est requis");
    }

    let resolved_in = super::tool_office_utils::resolve_path(input_path, working_dir);

    let validated_in = match validate_read_path(&resolved_in, working_dir) {
        Ok(p) => p,
        Err(e) => return ToolResult::err(e),
    };

    let resolved_out = super::tool_office_utils::resolve_path(output_path, working_dir);

    let validated_out = match validate_write_path(&resolved_out) {
        Ok(p) => p,
        Err(e) => return ToolResult::err(e),
    };

    if let Err(e) = ensure_source_size(&validated_in, MAX_IMAGE_SOURCE_BYTES, "Image") {
        return ToolResult::err(e);
    }
    if let Err(e) = validate_image_dimensions(&validated_in) {
        return ToolResult::err(e);
    }

    let mut img = match image::open(&validated_in) {
        Ok(i) => i,
        Err(_) => return ToolResult::err("Impossible d'ouvrir l'image"),
    };

    let ops = if operations.is_null() {
        vec![]
    } else {
        match super::tool_spreadsheet_write::coerce_to_array(operations) {
            Some(arr) => arr,
            None => return ToolResult::err("Le paramètre 'operations' doit être un tableau"),
        }
    };

    let mut quality: Option<u8> = None;

    for op in &ops {
        let op_type = op["type"].as_str().unwrap_or("");
        match op_type {
            "resize" => match apply_resize(img, op) {
                Ok(i) => img = i,
                Err(e) => return e,
            },
            "crop" => match apply_crop(img, op) {
                Ok(i) => img = i,
                Err(e) => return e,
            },
            "quality" => {
                let val = op["value"].as_u64().unwrap_or(85).clamp(1, 100) as u8;
                quality = Some(val);
            }
            unknown => {
                return ToolResult::err(format!("Opération inconnue: {unknown}"));
            }
        }
    }

    let (width, height) = (img.width(), img.height());

    let warning = webp_quality_warning(&validated_out, quality);
    if let Err(e) = save_image(&img, &validated_out, quality) {
        return ToolResult::err(e);
    }

    let file_size = match std::fs::metadata(&validated_out) {
        Ok(m) => m.len(),
        Err(_) => 0,
    };

    let json = serde_json::json!({
        "output_path": validated_out.to_string_lossy(),
        "width": width,
        "height": height,
        "file_size_bytes": file_size,
        "warning": warning
    });
    ToolResult::ok(json.to_string())
}

fn apply_resize(img: DynamicImage, op: &Value) -> Result<DynamicImage, ToolResult> {
    let w = bounded_dimension(&op["width"], "resize: 'width' requis")?;
    let h = bounded_dimension(&op["height"], "resize: 'height' requis")?;
    ensure_pixel_budget(w, h)?;
    let mode = op["mode"].as_str().unwrap_or("fit");

    let resized = match mode {
        "fill" => img.resize_to_fill(w, h, FilterType::Lanczos3),
        "exact" => img.resize_exact(w, h, FilterType::Lanczos3),
        _ => img.resize(w, h, FilterType::Lanczos3),
    };
    Ok(resized)
}

fn apply_crop(img: DynamicImage, op: &Value) -> Result<DynamicImage, ToolResult> {
    let x = bounded_coordinate(&op["x"], "crop: 'x' requis")?;
    let y = bounded_coordinate(&op["y"], "crop: 'y' requis")?;
    let w = bounded_dimension(&op["width"], "crop: 'width' requis")?;
    let h = bounded_dimension(&op["height"], "crop: 'height' requis")?;
    ensure_pixel_budget(w, h)?;
    if x.saturating_add(w) > img.width() || y.saturating_add(h) > img.height() {
        return Err(ToolResult::err("crop hors limites de l'image"));
    }

    Ok(img.crop_imm(x, y, w, h))
}

fn validate_image_dimensions(path: &Path) -> Result<(), String> {
    let reader = ImageReader::open(path)
        .map_err(|_| "Impossible d'ouvrir l'image".to_string())?
        .with_guessed_format()
        .map_err(|_| "Format image invalide".to_string())?;
    let (w, h) = reader
        .into_dimensions()
        .map_err(|_| "Impossible de lire les dimensions de l'image".to_string())?;
    if w == 0 || h == 0 || w > MAX_DIMENSION || h > MAX_DIMENSION {
        return Err("Dimensions image non supportées".to_string());
    }
    ensure_pixel_budget(w, h).map_err(|e| e.content)
}

fn bounded_dimension(value: &Value, missing: &str) -> Result<u32, ToolResult> {
    let raw = value.as_u64().ok_or_else(|| ToolResult::err(missing))?;
    let dimension = u32::try_from(raw).map_err(|_| ToolResult::err("Dimension trop grande"))?;
    if dimension == 0 || dimension > MAX_DIMENSION {
        return Err(ToolResult::err("Dimension hors limites"));
    }
    Ok(dimension)
}

fn bounded_coordinate(value: &Value, missing: &str) -> Result<u32, ToolResult> {
    let raw = value.as_u64().ok_or_else(|| ToolResult::err(missing))?;
    u32::try_from(raw).map_err(|_| ToolResult::err("Coordonnée trop grande"))
}

fn ensure_pixel_budget(w: u32, h: u32) -> Result<(), ToolResult> {
    let pixels = u64::from(w).saturating_mul(u64::from(h));
    if pixels > MAX_PIXELS {
        return Err(ToolResult::err("Image trop grande"));
    }
    Ok(())
}

fn webp_quality_warning(path: &Path, quality: Option<u8>) -> Option<&'static str> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    if quality.is_some() && ext == "webp" {
        Some("quality ignorée pour WebP lossless")
    } else {
        None
    }
}

fn save_image(img: &DynamicImage, path: &Path, quality: Option<u8>) -> Result<(), String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    if let Some(q) = quality {
        match ext.as_str() {
            "jpg" | "jpeg" => {
                let file = std::fs::File::create(path)
                    .map_err(|_| "Impossible de créer le fichier de sortie")?;
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(file, q);
                img.write_with_encoder(encoder)
                    .map_err(|_| "Erreur encodage JPEG")?;
                return Ok(());
            }
            "webp" => {
                // image 0.25 supporte uniquement WebP lossless — quality ignorée
                let file = std::fs::File::create(path)
                    .map_err(|_| "Impossible de créer le fichier de sortie")?;
                let encoder = image::codecs::webp::WebPEncoder::new_lossless(file);
                img.write_with_encoder(encoder)
                    .map_err(|_| "Erreur encodage WebP")?;
                return Ok(());
            }
            _ => {}
        }
    }

    img.save(path)
        .map_err(|_| "Erreur lors de la sauvegarde de l'image".to_string())
}
