use super::common::{rows, ExportBundle};
use super::quantile_labels::QuantileLabels;
use super::spreadsheet_text::safe_csv_cell;
use std::path::Path;

pub fn write(bundle: &ExportBundle, path: &Path) -> Result<(), String> {
    let mut writer =
        csv::Writer::from_path(path).map_err(|_| "Export CSV impossible".to_string())?;
    let labels = QuantileLabels::for_confidence(bundle.analysis.confidence_level);
    let [lower, median, upper] = labels.table_headers();
    writer
        .write_record([
            "section",
            "name",
            "date",
            "series_id",
            "value",
            lower,
            median,
            upper,
            "text",
            "source",
        ])
        .map_err(|_| "Export CSV impossible".to_string())?;
    for row in rows(bundle) {
        writer
            .write_record([
                row.section,
                safe_csv_cell(&row.name),
                safe_csv_cell(&row.date),
                safe_csv_cell(&row.series_id),
                row.value,
                row.q10,
                row.q50,
                row.q90,
                safe_csv_cell(&row.text),
                safe_csv_cell(&row.source),
            ])
            .map_err(|_| "Export CSV impossible".to_string())?;
    }
    writer
        .flush()
        .map_err(|_| "Export CSV impossible".to_string())
}
