use crate::services::agent_local::security::{validate_read_path, validate_write_path};
use crate::services::agent_local::tool_office_limits::{
    ensure_source_size, MAX_IMAGE_SOURCE_BYTES,
};
use crate::services::agent_local::types_tools::ToolResult;
use image::DynamicImage;
use serde_json::Value;
use std::path::Path;

pub async fn transform_image(
    input_path: &str,
    output_path: &str,
    operations: &Value,
    working_dir: &Path,
) -> ToolResult {
    if input_path.is_empty() {
        return ToolResult::validation(
            "image_input_path_required",
            "Le paramètre 'input_path' est requis",
        );
    }
    if operations.as_array().is_some_and(Vec::is_empty) && !output_path.is_empty() {
        return ToolResult::validation(
            "image_intent_ambiguous",
            "Demande ambiguë : pour inspecter, utilisez 'operations': [] sans 'output_path' ; pour convertir, fournissez 'output_path' et omettez 'operations'",
        );
    }
    let resolved_in = super::tool_office_utils::resolve_path(input_path, working_dir);

    let validated_in = match validate_read_path(&resolved_in, working_dir) {
        Ok(p) => p,
        Err(error) => {
            return super::tool_file_error::path_failure(
                error,
                "image_not_found",
                "image_read_denied",
                "invalid_image_input_path",
            )
        }
    };

    if let Err(e) = ensure_source_size(&validated_in, MAX_IMAGE_SOURCE_BYTES, "Image") {
        return ToolResult::validation("image_source_invalid", e);
    }
    if let Err(e) = super::tool_image_process_geometry::validate_dimensions(&validated_in) {
        return ToolResult::validation("image_dimensions_invalid", e);
    }

    let mut img = match image::open(&validated_in) {
        Ok(i) => i,
        Err(_) => return ToolResult::validation("image_content_invalid", "Impossible d'ouvrir l'image"),
    };

    if operations.as_array().is_some_and(Vec::is_empty) {
        return super::tool_image_inspect::inspect(&validated_in, &img);
    }

    if output_path.is_empty() {
        return ToolResult::validation(
            "image_output_path_required",
            "Le paramètre 'output_path' est requis pour transformer ou convertir une image",
        );
    }

    let resolved_out = super::tool_office_utils::resolve_path(output_path, working_dir);
    let validated_out = match validate_write_path(&resolved_out, working_dir) {
        Ok(p) => p,
        Err(error) => {
            return super::tool_file_error::path_failure(
                error,
                "image_parent_not_found",
                "image_write_denied",
                "invalid_image_output_path",
            )
        }
    };
    if let Err(error) =
        super::tool_image_process_geometry::validate_output_format(&validated_out)
    {
        return error;
    }

    let ops = if operations.is_null() {
        vec![]
    } else {
        match super::tool_office_array::coerce(
            operations,
            super::tool_office_limits::MAX_IMAGE_OPERATIONS,
        ) {
            Ok(operations) => operations,
            Err(super::tool_office_array::ArrayInputError::Invalid) => return ToolResult::validation(
                "image_operations_invalid",
                "Le paramètre 'operations' doit être un tableau",
            ),
            Err(super::tool_office_array::ArrayInputError::TooMany) => {
                return ToolResult::validation(
                    "image_operation_limit_exceeded",
                    format!(
                        "Trop d'opérations (maximum {})",
                        super::tool_office_limits::MAX_IMAGE_OPERATIONS
                    ),
                )
            }
        }
    };

    let mut quality: Option<u8> = None;

    for op in &ops {
        if !op.is_object() {
            return ToolResult::validation(
                "image_operation_invalid",
                "Chaque opération image doit être un objet",
            );
        }
        let Some(op_type) = op["type"].as_str() else {
            return ToolResult::validation(
                "image_operation_type_required",
                "Chaque opération image doit définir 'type'",
            );
        };
        match op_type {
            "resize" => match super::tool_image_process_geometry::resize(img, op) {
                Ok(i) => img = i,
                Err(e) => return e,
            },
            "crop" => match super::tool_image_process_geometry::crop(img, op) {
                Ok(i) => img = i,
                Err(e) => return e,
            },
            "quality" => {
                quality = match super::tool_image_process_geometry::quality(op) {
                    Ok(value) => Some(value),
                    Err(error) => return error,
                };
            }
            unknown => {
                return ToolResult::validation(
                    "image_operation_unsupported",
                    format!("Opération inconnue: {unknown}"),
                );
            }
        }
    }

    let (width, height) = (img.width(), img.height());

    let quality_warning = quality_warning(&validated_out, quality);
    if let Err(e) = save_image(&img, &validated_out, quality) {
        return ToolResult::execution("image_write_failed", e, false).with_error_hint(
            "Vérifier le fichier cible avant une nouvelle écriture : il peut être partiel.",
        );
    }

    let (file_size, metadata_warning) = match std::fs::metadata(&validated_out) {
        Ok(metadata) => (metadata.len(), None),
        Err(_) => (0, Some("La taille du fichier de sortie n'a pas pu être confirmée.")),
    };

    let json = serde_json::json!({
        "output_path": validated_out.to_string_lossy(),
        "width": width,
        "height": height,
        "file_size_bytes": file_size,
        "warning": quality_warning
    });
    let mut result = ToolResult::ok(json.to_string());
    if let Some(warning) = quality_warning {
        result = result.with_warning(warning);
    }
    if let Some(warning) = metadata_warning {
        result = result.with_warning(warning);
    }
    result
}

fn quality_warning(path: &Path, quality: Option<u8>) -> Option<&'static str> {
    quality?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "jpg" | "jpeg" => None,
        "webp" => Some("quality ignorée pour WebP lossless"),
        _ => Some("quality ignorée pour ce format de sortie"),
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
