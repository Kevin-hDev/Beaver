use super::chart_data::{Chart, B, H, L, R, W};
use super::common::ExportBundle;
use resvg::{tiny_skia, usvg};
use std::path::Path;

pub fn write_svg(bundle: &ExportBundle, path: &Path) -> Result<(), String> {
    std::fs::write(path, svg_string(bundle)).map_err(|_| "Export SVG impossible".to_string())
}

pub fn write_png(bundle: &ExportBundle, path: &Path) -> Result<(), String> {
    let svg = svg_string(bundle);
    let mut options = usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    let tree = usvg::Tree::from_data(svg.as_bytes(), &options)
        .map_err(|_| "Export PNG impossible".to_string())?;
    let mut pixmap =
        tiny_skia::Pixmap::new(W, H).ok_or_else(|| "Export PNG impossible".to_string())?;
    resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    pixmap
        .save_png(path)
        .map_err(|_| "Export PNG impossible".to_string())
}

fn svg_string(bundle: &ExportBundle) -> String {
    let chart = Chart::from(bundle);
    let target = &bundle.analysis.target_column;
    let subtitle = format!(
        "{} | {} | H{} | {} points",
        bundle.analysis.model,
        target,
        bundle.analysis.horizon,
        bundle.analysis.predictions.len()
    );
    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">
<rect width="100%" height="100%" fill="#101012"/>
<rect x="24" y="24" width="1352" height="712" rx="24" fill="#171719" stroke="#29292e"/>
<text x="54" y="64" fill="#d7d7dc" font-family="Inter, Arial, sans-serif" font-size="24" font-weight="700">{}</text>
<text x="54" y="92" fill="#8f8fa3" font-family="Inter, Arial, sans-serif" font-size="15">{}</text>"##,
        escape(&bundle.analysis.name),
        escape(&subtitle)
    );
    svg.push_str(&legend());
    svg.push_str(&grid_and_axes(&chart, target));
    if !chart.band.is_empty() {
        svg.push_str(&band_path(&chart));
    }
    svg.push_str(&line_path(&chart, &chart.history, "#cfd0d6", 4.0, false));
    svg.push_str(&line_path(&chart, &chart.prediction, "#ff6a00", 4.0, false));
    svg.push_str(&markers(&chart, &chart.prediction, "#ff6a00"));
    for scenario in &chart.scenarios {
        svg.push_str(&line_path(&chart, scenario, "#5c9cff", 2.2, true));
    }
    svg.push_str("</svg>");
    svg
}

fn grid_and_axes(chart: &Chart, target: &str) -> String {
    let mut out = String::new();
    let right = W as f64 - R;
    let bottom = H as f64 - B;
    for (value, y) in chart.y_ticks() {
        out.push_str(&format!(
            r##"<line x1="{L}" y1="{y:.1}" x2="{right}" y2="{y:.1}" stroke="#29292e" stroke-width="1"/>
<text x="72" y="{:.1}" fill="#8f8fa3" font-family="Inter, Arial, sans-serif" font-size="15" text-anchor="end">{}</text>"##,
            y + 5.0,
            escape(&axis_value(value, target))
        ));
    }
    out.push_str(&format!(
        r##"<line x1="{L}" y1="{bottom}" x2="{right}" y2="{bottom}" stroke="#35353b" stroke-width="1.5"/>"##
    ));
    for (label, x) in chart.labels() {
        out.push_str(&format!(
            r##"<text x="{x:.1}" y="680" fill="#8f8fa3" font-family="Inter, Arial, sans-serif" font-size="18" text-anchor="middle">{}</text>"##,
            escape(&label)
        ));
    }
    out
}

fn legend() -> String {
    [
        legend_item(1030.0, "Historique", "#cfd0d6"),
        legend_item(1165.0, "Prevision", "#ff6a00"),
        legend_item(1290.0, "Confiance", "#5a3020"),
    ]
    .join("")
}

fn legend_item(x: f64, label: &str, color: &str) -> String {
    format!(
        r##"<circle cx="{x}" cy="67" r="5" fill="{color}"/>
<text x="{}" y="73" fill="#b8b8c2" font-family="Inter, Arial, sans-serif" font-size="14">{}</text>"##,
        x + 14.0,
        escape(label)
    )
}

fn line_path(
    chart: &Chart,
    points: &[(f64, f64)],
    color: &str,
    width: f64,
    dashed: bool,
) -> String {
    if points.is_empty() {
        return String::new();
    }
    let d = points
        .iter()
        .enumerate()
        .map(|(i, (x, y))| {
            format!(
                "{} {:.1} {:.1}",
                if i == 0 { "M" } else { "L" },
                chart.x(*x),
                chart.y(*y)
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let dash = if dashed {
        r#" stroke-dasharray="8 8""#
    } else {
        ""
    };
    format!(
        r##"<path d="{d}" fill="none" stroke="{color}" stroke-width="{width}" stroke-linecap="round" stroke-linejoin="round"{dash}/>"##
    )
}

fn band_path(chart: &Chart) -> String {
    let mut d = String::new();
    for (i, (x, low, _)) in chart.band.iter().enumerate() {
        d.push_str(&format!(
            "{} {:.1} {:.1} ",
            if i == 0 { "M" } else { "L" },
            chart.x(*x),
            chart.y(*low)
        ));
    }
    for (x, _, high) in chart.band.iter().rev() {
        d.push_str(&format!("L {:.1} {:.1} ", chart.x(*x), chart.y(*high)));
    }
    format!(r##"<path d="{d}Z" fill="#ff6a00" opacity="0.12"/>"##)
}

fn markers(chart: &Chart, points: &[(f64, f64)], color: &str) -> String {
    points
        .iter()
        .map(|(x, y)| {
            format!(
                r##"<circle cx="{:.1}" cy="{:.1}" r="5.5" fill="#171719" stroke="{color}" stroke-width="4"/>"##,
                chart.x(*x),
                chart.y(*y)
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn axis_value(value: f64, target: &str) -> String {
    let suffix = if target.to_ascii_lowercase().contains("eur") {
        " €"
    } else {
        ""
    };
    if value.abs() >= 1000.0 {
        format!("{:.1} k{suffix}", value / 1000.0)
    } else {
        format!("{value:.0}{suffix}")
    }
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
