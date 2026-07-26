use super::stats;
use super::{AnalyticsStatus, ResidualAnomaly, SeriesDecomposition};

const MAX_ANOMALIES: usize = 100;
const ROBUST_THRESHOLD: f64 = 3.5;

pub(super) fn detect(decompositions: &[SeriesDecomposition]) -> Vec<ResidualAnomaly> {
    let mut anomalies = Vec::new();
    for decomposition in decompositions {
        if decomposition.status != AnalyticsStatus::Ready {
            continue;
        }
        let all: Vec<_> = decomposition
            .points
            .iter()
            .map(|point| point.residual)
            .collect();
        let global_center = stats::median(&all);
        let global_mad = stats::mad(&all, global_center);
        let mut phases = vec![Vec::new(); decomposition.period];
        for (index, residual) in all.iter().enumerate() {
            phases[index % decomposition.period].push(*residual);
        }
        for (index, point) in decomposition.points.iter().enumerate() {
            let phase = &phases[index % decomposition.period];
            let phase_center = stats::median(phase);
            let phase_mad = stats::mad(phase, phase_center);
            let (reference, center, mad) = if phase.len() >= 4 && phase_mad > f64::EPSILON {
                (phase.as_slice(), phase_center, phase_mad)
            } else {
                (all.as_slice(), global_center, global_mad)
            };
            let scale = if mad > f64::EPSILON {
                mad / 0.674_489_75
            } else {
                stats::variance(reference).sqrt()
            };
            if scale <= f64::EPSILON {
                continue;
            }
            let score = ((point.residual - center) / scale).abs();
            if score < ROBUST_THRESHOLD {
                continue;
            }
            anomalies.push(ResidualAnomaly {
                id: format!(
                    "{}:{}",
                    decomposition.series_id.as_deref().unwrap_or("series-1"),
                    index
                ),
                series_id: decomposition.series_id.clone(),
                date: point.date.clone(),
                observed: point.observed,
                expected: point.trend + point.seasonal,
                residual: point.residual,
                score,
                severity: if score >= 6.0 { "high" } else { "medium" }.into(),
                method: "seasonal_robust_residual".into(),
            });
        }
    }
    anomalies.sort_by(|left, right| right.score.total_cmp(&left.score));
    anomalies.truncate(MAX_ANOMALIES);
    anomalies
}
