//! Tests des helpers d'export PURE : safe_file_stem (sécurité nom de fichier),
//! fmt (formatage nombre), clipboard_text et rows (avec bundle minimal).

use super::common::{clipboard_text, fmt, rows, safe_file_stem, ExportBundle};
use crate::services::forecast::types::ForecastResult;

// --- safe_file_stem (sécurité nom de fichier) -------------------------------

#[test]
fn safe_file_stem_keeps_alphanumeric() {
    assert_eq!(safe_file_stem("Sales 2026"), "sales-2026");
}

#[test]
fn safe_file_stem_lowercases() {
    assert_eq!(safe_file_stem("ForecastQ1"), "forecastq1");
}

#[test]
fn safe_file_stem_replaces_special_chars() {
    // Les caractères spéciaux dangereux pour un filesystem sont supprimés.
    assert_eq!(safe_file_stem("a/b\\c?d*e"), "abcde");
    assert_eq!(safe_file_stem("café"), "caf"); // accents supprimés
}

#[test]
fn safe_file_stem_collapses_multiple_dashes() {
    assert_eq!(safe_file_stem("a   b---c"), "a-b-c");
    assert_eq!(safe_file_stem("---"), "forecast"); // que des tirets → default
}

#[test]
fn safe_file_stem_truncates_to_64_chars() {
    let long = "a".repeat(100);
    let result = safe_file_stem(&long);
    assert!(result.len() <= 64, "ne doit pas dépasser 64 caractères");
}

#[test]
fn safe_file_stem_empty_returns_default() {
    assert_eq!(safe_file_stem(""), "forecast");
    assert_eq!(safe_file_stem("   "), "forecast");
    assert_eq!(safe_file_stem("!!!"), "forecast");
}

#[test]
fn safe_file_stem_path_traversal_safe() {
    // Sécurité : aucun `..` ou `/` ne doit survivre (pas de traversal).
    let result = safe_file_stem("../../etc/passwd");
    assert!(!result.contains('.'));
    assert!(!result.contains('/'));
    assert!(!result.contains('\\'));
}

#[test]
fn safe_file_stem_preserves_dashes_and_underscores() {
    assert_eq!(safe_file_stem("my_forecast-1"), "my-forecast-1");
}

// --- fmt (formatage nombre) -------------------------------------------------

#[test]
fn fmt_formats_finite_numbers() {
    assert_eq!(fmt(42.56789), "42.567890");
    assert_eq!(fmt(100.0), "100.000000");
    assert_eq!(fmt(0.5), "0.500000");
}

#[test]
fn fmt_returns_empty_for_non_finite() {
    assert_eq!(fmt(f64::NAN), "");
    assert_eq!(fmt(f64::INFINITY), "");
    assert_eq!(fmt(f64::NEG_INFINITY), "");
}

// --- clipboard_text et rows avec bundle minimal -----------------------------

/// Construit un ForecastResult minimal via serde (les champs default rendent
/// la construction tolérante).
pub(super) fn minimal_result() -> ForecastResult {
    serde_json::from_value(serde_json::json!({
        "id": "test-1",
        "name": "Test Forecast",
        "target_column": "sales",
        "created_at": "2026-01-01T00:00:00Z",
        "model": "chronos",
        "provider": "local",
        "horizon": 3,
        "frequency": "D",
        "input_summary": {"points": 2, "start": "2026-01-01", "end": "2026-01-02"},
        "predictions": [
            {"date": "2026-01-03", "value": 10.5},
            {"date": "2026-01-04", "value": 12.0},
            {"date": "2026-01-05", "value": 14.5}
        ],
        "quantiles": {
            "q10": [8.0, 9.0, 11.0],
            "q50": [10.5, 12.0, 14.5],
            "q90": [13.0, 15.0, 18.0]
        }
    }))
    .expect("deserialize minimal result")
}

fn minimal_bundle() -> ExportBundle {
    ExportBundle {
        analysis: minimal_result(),
        notes: vec![],
    }
}

#[test]
fn clipboard_text_includes_header_and_predictions() {
    let bundle = minimal_bundle();
    let text = clipboard_text(&bundle);

    assert!(text.contains("Forecast: Test Forecast"));
    assert!(text.contains("Model: chronos"));
    assert!(text.contains("Target: sales"));
    assert!(text.contains("2026-01-03"));
    assert!(text.contains("10.500000"));
    // En-tête TSV
    assert!(text.contains("date\tseries\tprediction\tq05\tq50\tq95"));
}

#[test]
fn rows_generates_prediction_rows() {
    let bundle = minimal_bundle();
    let table = rows(&bundle);

    let prediction_rows: Vec<_> = table.iter().filter(|r| r.section == "prediction").collect();
    assert_eq!(
        prediction_rows.len(),
        3,
        "doit générer une ligne par prédiction"
    );
    assert_eq!(prediction_rows[0].date, "2026-01-03");
    assert_eq!(prediction_rows[0].value, "10.500000");
    assert_eq!(prediction_rows[0].q10, "8.000000");
    assert_eq!(prediction_rows[0].q90, "13.000000");
}

#[test]
fn rows_empty_bundle_returns_empty() {
    // Bundle sans prédictions/notes/scenarios → aucune ligne générée.
    let analysis: ForecastResult = serde_json::from_value(serde_json::json!({
        "id": "x",
        "name": "x",
        "created_at": "x",
        "model": "x",
        "provider": "x",
        "horizon": 0,
        "frequency": "D",
        "input_summary": {"points": 0, "start": "", "end": ""},
        "predictions": [],
        "quantiles": {}
    }))
    .expect("deserialize empty result");
    let bundle = ExportBundle {
        analysis,
        notes: vec![],
    };

    let table = rows(&bundle);
    assert!(table.is_empty());
}
