use super::common::ExportBundle;
use super::{common_tests, pdf};

#[test]
fn pdf_uses_the_beaver_public_name() {
    let bundle = ExportBundle {
        analysis: common_tests::minimal_result(),
        notes: Vec::new(),
    };
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("forecast.pdf");

    pdf::write(&bundle, &path).expect("write PDF");
    let content = std::fs::read_to_string(path).expect("read PDF");

    assert!(content.contains("Beaver Forecast"));
    assert!(!content.contains("CL-GO"));
}
