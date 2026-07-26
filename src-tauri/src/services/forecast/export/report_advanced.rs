use super::common::ExportBundle;

pub(super) fn lines(bundle: &ExportBundle) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(advanced) = &bundle.analysis.advanced_analytics {
        lines.push(String::new());
        lines.push("ANALYSE AVANCEE".into());
        for item in &advanced.decomposition {
            lines.push(format!(
                "Decomposition {:<12} methode={} periode={} force={}",
                short(item.series_id.as_deref().unwrap_or("serie-1"), 12),
                item.method,
                item.period,
                optional(item.seasonal_strength)
            ));
        }
        lines.push(format!(
            "Anomalies residuelles {}",
            advanced.anomalies.len()
        ));
        for item in advanced.anomalies.iter().take(10) {
            lines.push(format!(
                "  {} | {} | score {:.2} | observe {:.2} attendu {:.2}",
                short(&item.date, 20),
                short(item.series_id.as_deref().unwrap_or("serie-1"), 12),
                item.score,
                item.observed,
                item.expected
            ));
        }
        lines.push(format!(
            "Importance variables methode={} fiabilite={}",
            advanced.variable_importance.method, advanced.variable_importance.reliability
        ));
        for item in advanced.variable_importance.items.iter().take(10) {
            lines.push(format!(
                "  {:<24} {:>6.1}% {}",
                short(&item.name, 24),
                item.normalized_score * 100.0,
                item.direction
            ));
        }
        for item in &advanced.drift {
            lines.push(format!(
                "Derive {:<12} detectee={} score={} severite={}",
                short(item.series_id.as_deref().unwrap_or("serie-1"), 12),
                item.detected,
                optional(item.score),
                item.severity
            ));
        }
    }
    append_evaluation(bundle, &mut lines);
    append_ensemble(bundle, &mut lines);
    lines
}

fn append_evaluation(bundle: &ExportBundle, lines: &mut Vec<String>) {
    let Some(evaluation) = &bundle.analysis.evaluation else {
        return;
    };
    lines.push(String::new());
    lines.push(format!("BACKTEST {} fenetres", evaluation.windows));
    let mut results: Vec<_> = evaluation.results.iter().collect();
    results.sort_by_key(|result| result.rank.unwrap_or(usize::MAX));
    for result in results {
        lines.push(match &result.metrics {
            Some(metrics) => format!(
                "  #{} {:<28} MASE {:.4} sMAPE {:.2} MAE {:.4}",
                result.rank.unwrap_or(0),
                short(&result.model_id, 28),
                metrics.mase,
                metrics.smape,
                metrics.mae
            ),
            None => format!("  - {:<28} echec", short(&result.model_id, 28)),
        });
    }
}

fn append_ensemble(bundle: &ExportBundle, lines: &mut Vec<String>) {
    let Some(ensemble) = &bundle.analysis.ensemble else {
        return;
    };
    lines.push(String::new());
    lines.push(format!("ENSEMBLE {}", ensemble.validation_status));
    for member in &ensemble.members {
        lines.push(format!(
            "  {:<28} poids {:>6.1}% MASE {:.4}",
            short(&member.model_id, 28),
            member.weight * 100.0,
            member.backtest_mase
        ));
    }
}

fn optional(value: Option<f64>) -> String {
    value.map_or_else(|| "n/a".into(), |item| format!("{item:.3}"))
}

fn short(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        value.to_string()
    } else {
        value
            .chars()
            .take(max.saturating_sub(3))
            .chain("...".chars())
            .collect()
    }
}
