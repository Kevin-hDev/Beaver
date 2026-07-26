mod advanced_rows;
#[cfg(test)]
mod advanced_rows_tests;
mod chart;
mod chart_data;
mod common;
#[cfg(test)]
mod common_tests;
mod csv;
#[cfg(test)]
mod csv_tests;
mod pdf;
#[cfg(test)]
mod pdf_tests;
mod quantile_labels;
mod report_advanced;
mod spreadsheet_text;
mod xlsx;
mod xlsx_advanced;
mod xlsx_input;
#[cfg(test)]
mod xlsx_security_tests;
mod xlsx_style;

use super::{notes, storage};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastExportKind {
    File,
    Clipboard,
}

#[derive(Debug, Serialize)]
pub struct ForecastExportResult {
    pub kind: ForecastExportKind,
    pub format: String,
    pub file_path: Option<String>,
    pub content: Option<String>,
}

pub async fn export_analysis(
    analysis_id: &str,
    format: &str,
) -> Result<ForecastExportResult, String> {
    let format = ExportFormat::parse(format)?;
    let notes = notes::list(analysis_id).await?.notes;
    let analysis = storage::load(analysis_id).await?;
    let bundle = common::ExportBundle { analysis, notes };

    if format == ExportFormat::Clipboard {
        return Ok(ForecastExportResult {
            kind: ForecastExportKind::Clipboard,
            format: format.extension().into(),
            file_path: None,
            content: Some(common::clipboard_text(&bundle)),
        });
    }

    let path = output_path(
        &bundle.analysis.id,
        &bundle.analysis.name,
        format.extension(),
    )?;
    match format {
        ExportFormat::Csv => csv::write(&bundle, &path)?,
        ExportFormat::Excel => xlsx::write(&bundle, &path)?,
        ExportFormat::Json => common::write_json(&bundle, &path)?,
        ExportFormat::Svg => chart::write_svg(&bundle, &path)?,
        ExportFormat::Png => chart::write_png(&bundle, &path)?,
        ExportFormat::Pdf => pdf::write(&bundle, &path)?,
        ExportFormat::Clipboard => {}
    }

    Ok(ForecastExportResult {
        kind: ForecastExportKind::File,
        format: format.extension().into(),
        file_path: Some(path.to_string_lossy().to_string()),
        content: None,
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExportFormat {
    Csv,
    Excel,
    Json,
    Png,
    Svg,
    Pdf,
    Clipboard,
}

impl ExportFormat {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "csv" => Ok(Self::Csv),
            "excel" | "xlsx" => Ok(Self::Excel),
            "json" => Ok(Self::Json),
            "png" => Ok(Self::Png),
            "svg" => Ok(Self::Svg),
            "pdf" => Ok(Self::Pdf),
            "clipboard" => Ok(Self::Clipboard),
            _ => Err("Format d'export invalide".into()),
        }
    }

    fn extension(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Excel => "xlsx",
            Self::Json => "json",
            Self::Png => "png",
            Self::Svg => "svg",
            Self::Pdf => "pdf",
            Self::Clipboard => "clipboard",
        }
    }
}

fn output_path(id: &str, name: &str, extension: &str) -> Result<PathBuf, String> {
    let dir = dirs::download_dir()
        .unwrap_or_else(|| crate::services::paths::data_dir().join("forecast-exports"));
    std::fs::create_dir_all(&dir)
        .map_err(|_| "Impossible de créer le dossier d'export".to_string())?;
    let safe_name = common::safe_file_stem(name);
    let short_id: String = id.chars().take(8).collect();
    Ok(dir.join(format!("{safe_name}-{short_id}.{extension}")))
}
