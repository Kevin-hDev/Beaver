use super::common::ExportBundle;

pub const W: u32 = 1400;
pub const H: u32 = 760;
pub const L: f64 = 90.0;
pub const R: f64 = 50.0;
pub const T: f64 = 110.0;
pub const B: f64 = 120.0;

pub struct Chart {
    pub history: Vec<(f64, f64)>,
    pub prediction: Vec<(f64, f64)>,
    pub scenarios: Vec<Vec<(f64, f64)>>,
    pub band: Vec<(f64, f64, f64)>,
    min: f64,
    max: f64,
    dates: Vec<String>,
}

impl Chart {
    pub fn from(bundle: &ExportBundle) -> Self {
        let history_len = bundle.analysis.input_data.history.len();
        let history = points_from(&bundle.analysis.input_data.history, 0);
        let prediction = points_from(&bundle.analysis.predictions, history_len);
        let scenarios: Vec<Vec<(f64, f64)>> = bundle
            .analysis
            .scenarios
            .iter()
            .map(|s| points_from(&s.predictions, history_len))
            .collect();
        let band = band_from(bundle, history_len);
        let dates = dates_from(bundle);
        let (min, max) = bounds(&history, &prediction, &scenarios, &band);
        Self {
            history,
            prediction,
            scenarios,
            band,
            min,
            max,
            dates,
        }
    }

    pub fn x(&self, idx: f64) -> f64 {
        let max_idx = (self.dates.len().saturating_sub(1)).max(1) as f64;
        L + idx / max_idx * (W as f64 - L - R)
    }

    pub fn y(&self, value: f64) -> f64 {
        let span = (self.max - self.min).max(1.0);
        T + (self.max - value) / span * (H as f64 - T - B)
    }

    pub fn labels(&self) -> Vec<(String, f64)> {
        if self.dates.is_empty() {
            return Vec::new();
        }
        let last = self.dates.len() - 1;
        let mut indexes = [0, last / 4, last / 2, last * 3 / 4, last]
            .into_iter()
            .collect::<Vec<_>>();
        indexes.dedup();
        indexes
            .into_iter()
            .map(|idx| (short_date(&self.dates[idx]), self.x(idx as f64)))
            .collect()
    }

    pub fn y_ticks(&self) -> Vec<(f64, f64)> {
        (0..=5)
            .map(|idx| {
                let value = self.min + (self.max - self.min) * idx as f64 / 5.0;
                (value, self.y(value))
            })
            .collect()
    }
}

fn points_from(points: &[super::super::types::Prediction], offset: usize) -> Vec<(f64, f64)> {
    points
        .iter()
        .enumerate()
        .map(|(i, p)| ((offset + i) as f64, p.value))
        .collect()
}

fn band_from(bundle: &ExportBundle, offset: usize) -> Vec<(f64, f64, f64)> {
    bundle
        .analysis
        .predictions
        .iter()
        .enumerate()
        .filter_map(|(i, _)| {
            Some((
                (offset + i) as f64,
                *bundle.analysis.quantiles.q10.get(i)?,
                *bundle.analysis.quantiles.q90.get(i)?,
            ))
        })
        .collect()
}

fn dates_from(bundle: &ExportBundle) -> Vec<String> {
    let mut dates: Vec<String> = bundle
        .analysis
        .input_data
        .history
        .iter()
        .map(|p| p.date.clone())
        .collect();
    dates.extend(bundle.analysis.predictions.iter().map(|p| p.date.clone()));
    dates
}

fn bounds(
    history: &[(f64, f64)],
    prediction: &[(f64, f64)],
    scenarios: &[Vec<(f64, f64)>],
    band: &[(f64, f64, f64)],
) -> (f64, f64) {
    let mut values = Vec::new();
    collect_values(&mut values, history);
    collect_values(&mut values, prediction);
    for scenario in scenarios {
        collect_values(&mut values, scenario);
    }
    for (_, low, high) in band {
        values.push(*low);
        values.push(*high);
    }
    min_max(&values)
}

fn collect_values(values: &mut Vec<f64>, points: &[(f64, f64)]) {
    values.extend(
        points
            .iter()
            .map(|(_, value)| *value)
            .filter(|value| value.is_finite()),
    );
}

fn min_max(values: &[f64]) -> (f64, f64) {
    let min = values
        .iter()
        .fold(f64::INFINITY, |acc, value| acc.min(*value));
    let max = values
        .iter()
        .fold(f64::NEG_INFINITY, |acc, value| acc.max(*value));
    if !min.is_finite() || !max.is_finite() {
        return (0.0, 1.0);
    }
    let pad = ((max - min) * 0.1).max(1.0);
    (min - pad, max + pad)
}

fn short_date(date: &str) -> String {
    date.get(5..10).unwrap_or(date).to_string()
}
