use super::common::ExportBundle;
use crate::services::forecast::notes::ForecastNote;
use crate::services::forecast::types::ForecastResult;

#[test]
fn formula_like_note_fields_are_exported_as_text() {
    let bundle = ExportBundle {
        analysis: analysis(),
        notes: vec![ForecastNote {
            id: "550e8400-e29b-41d4-a716-446655440001".into(),
            analysis_id: "550e8400-e29b-41d4-a716-446655440000".into(),
            date: "2026-07-23".into(),
            title: "=IMPORTXML(\"https://example.com\")".into(),
            note_type: "context".into(),
            source: "user".into(),
            content: "+SUM(1,1)".into(),
            file_path: String::new(),
            created_at: "2026-07-23T00:00:00Z".into(),
            updated_at: "2026-07-23T00:00:00Z".into(),
        }],
    };
    let file = tempfile::NamedTempFile::new().unwrap();

    super::csv::write(&bundle, file.path()).unwrap();

    let records: Vec<_> = csv::Reader::from_path(file.path())
        .unwrap()
        .records()
        .map(Result::unwrap)
        .collect();
    let note = records
        .iter()
        .find(|record| record.get(0) == Some("note"))
        .unwrap();
    assert!(note.get(1).unwrap().starts_with("'="));
    assert!(note.get(8).unwrap().starts_with("'+"));
}

fn analysis() -> ForecastResult {
    serde_json::from_value(serde_json::json!({
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "Export sûr",
        "created_at": "2026-07-23T00:00:00Z",
        "model": "naive",
        "provider": "local",
        "horizon": 1,
        "frequency": "D",
        "input_summary": {"points": 2, "start": "2026-07-21", "end": "2026-07-22"},
        "predictions": [{"date": "2026-07-23", "value": 1.0}],
        "quantiles": {"q10": [0.5], "q50": [1.0], "q90": [1.5]}
    }))
    .unwrap()
}
