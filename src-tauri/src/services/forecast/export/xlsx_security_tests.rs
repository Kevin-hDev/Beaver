use super::common::ExportBundle;
use crate::services::forecast::notes::ForecastNote;
use std::io::Read;

#[test]
fn formula_like_note_values_remain_xlsx_strings() {
    let mut bundle = ExportBundle {
        analysis: super::common_tests::minimal_result(),
        notes: Vec::new(),
    };
    bundle.notes.push(ForecastNote {
        id: "550e8400-e29b-41d4-a716-446655440001".into(),
        analysis_id: bundle.analysis.id.clone(),
        date: "2026-07-23".into(),
        title: "=HYPERLINK(\"https://example.invalid\")".into(),
        note_type: "context".into(),
        source: "user".into(),
        content: "+SUM(1,1)".into(),
        file_path: String::new(),
        created_at: "2026-07-23T00:00:00Z".into(),
        updated_at: "2026-07-23T00:00:00Z".into(),
    });
    let file = tempfile::NamedTempFile::new().unwrap();

    super::xlsx::write(&bundle, file.path()).unwrap();

    let mut archive = zip::ZipArchive::new(std::fs::File::open(file.path()).unwrap()).unwrap();
    let mut shared_strings = String::new();
    archive
        .by_name("xl/sharedStrings.xml")
        .unwrap()
        .read_to_string(&mut shared_strings)
        .unwrap();
    assert!(shared_strings.contains("=HYPERLINK"));
    assert!(shared_strings.contains("+SUM(1,1)"));

    let names: Vec<String> = (0..archive.len())
        .map(|index| archive.by_index(index).unwrap().name().to_string())
        .collect();
    for name in names
        .iter()
        .filter(|name| name.starts_with("xl/worksheets/sheet"))
    {
        let mut sheet = String::new();
        archive
            .by_name(name)
            .unwrap()
            .read_to_string(&mut sheet)
            .unwrap();
        assert!(
            !sheet.contains("<f>"),
            "formule XLSX inattendue dans {name}"
        );
    }
}
