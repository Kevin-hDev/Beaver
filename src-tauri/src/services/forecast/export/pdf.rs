use super::common::ExportBundle;
use super::quantile_labels::QuantileLabels;
use crate::services::brand::DISPLAY_NAME;
use std::path::Path;

const LINES_PER_PAGE: usize = 46;

pub fn write(bundle: &ExportBundle, path: &Path) -> Result<(), String> {
    let lines = report_lines(bundle);
    let pages: Vec<&[String]> = lines.chunks(LINES_PER_PAGE).collect();
    let mut objects = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        pages_object(pages.len()),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Courier >>".to_string(),
    ];
    for (idx, page_lines) in pages.iter().enumerate() {
        let page_obj = 4 + idx * 2;
        let content_obj = page_obj + 1;
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 3 0 R >> >> /Contents {content_obj} 0 R >>"
        ));
        objects.push(stream_object(&content_stream(page_lines)));
    }
    let bytes = build_pdf(objects);
    std::fs::write(path, bytes).map_err(|_| "Export PDF impossible".to_string())
}

fn report_lines(bundle: &ExportBundle) -> Vec<String> {
    let a = &bundle.analysis;
    let labels = QuantileLabels::for_confidence(a.confidence_level);
    let [lower, median, upper] = labels.uppercase_headers();
    let mut lines = vec![
        format!("{DISPLAY_NAME} Forecast"),
        format!("Analyse      {}", a.name),
        format!("Modele       {} ({})", a.model, a.provider),
        format!("Cible        {}", a.target_column),
        format!(
            "Horizon      {} points | Frequence {}",
            a.horizon, a.frequency
        ),
        format!(
            "Historique   {} -> {}",
            a.input_summary.start, a.input_summary.end
        ),
        format!("Points       {}", a.input_summary.points),
        String::new(),
        "PREVISIONS".into(),
        format!(
            "{:<20} {:<12} {:>12} {:>12} {:>12} {:>12}",
            "Date", "Serie", "Valeur", lower, median, upper
        ),
        "-".repeat(88),
    ];
    for (idx, point) in a.predictions.iter().enumerate() {
        lines.push(format!(
            "{:<20} {:<12} {:>12} {:>12} {:>12} {:>12}",
            short(&point.date, 20),
            short(point.series_id.as_deref().unwrap_or(""), 12),
            money(point.value),
            q(&a.quantiles.q10, idx),
            q(&a.quantiles.q50, idx),
            q(&a.quantiles.q90, idx)
        ));
    }
    lines.extend(super::report_advanced::lines(bundle));
    if !a.scenarios.is_empty() {
        lines.push(String::new());
        lines.push("SCENARIOS".into());
        for scenario in &a.scenarios {
            lines.push(format!(
                "{:<42} {:>8} points",
                short(&scenario.name, 42),
                scenario.predictions.len()
            ));
        }
    }
    if !bundle.notes.is_empty() {
        lines.push(String::new());
        lines.push("NOTES".into());
        for note in &bundle.notes {
            lines.push(format!(
                "{} | {} | {}",
                short(&note.date, 16),
                short(&note.note_type, 14),
                short(&note.title, 54)
            ));
            for line in note.content.lines().take(4) {
                lines.push(format!("  {}", short(line, 84)));
            }
        }
    }
    lines
}

fn pages_object(count: usize) -> String {
    let kids = (0..count)
        .map(|idx| format!("{} 0 R", 4 + idx * 2))
        .collect::<Vec<_>>()
        .join(" ");
    format!("<< /Type /Pages /Kids [{kids}] /Count {count} >>")
}

fn stream_object(stream: &str) -> String {
    format!(
        "<< /Length {} >>\nstream\n{}\nendstream",
        stream.len(),
        stream
    )
}

fn content_stream(lines: &[String]) -> String {
    let mut body = String::from("BT /F1 9 Tf 46 800 Td 13 TL ");
    for line in lines {
        body.push_str(&format!("({}) Tj T* ", pdf_escape(line)));
    }
    body.push_str("ET");
    body
}

fn build_pdf(objects: Vec<String>) -> Vec<u8> {
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::with_capacity(objects.len());
    for (idx, object) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", idx + 1, object).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

fn pdf_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

fn q(values: &[f64], idx: usize) -> String {
    values.get(idx).map(|v| money(*v)).unwrap_or_default()
}

fn money(value: f64) -> String {
    if value.is_finite() {
        format!("{:.2}", value)
    } else {
        String::new()
    }
}

fn short(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        value.to_string()
    } else {
        let keep = max.saturating_sub(1);
        format!("{}...", value.chars().take(keep).collect::<String>())
    }
}
