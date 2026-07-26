#[derive(Debug, Clone, Copy)]
pub(super) enum Baseline {
    Naive,
    SeasonalNaive,
    Drift,
    Ets,
}

impl Baseline {
    pub(super) const ALL: [Self; 4] = [Self::Naive, Self::SeasonalNaive, Self::Drift, Self::Ets];

    pub(super) fn id(self) -> &'static str {
        match self {
            Self::Naive => "naive",
            Self::SeasonalNaive => "seasonal_naive",
            Self::Drift => "drift",
            Self::Ets => "ets",
        }
    }

    pub(super) fn forecast(
        self,
        training: &[f64],
        horizon: usize,
        period: usize,
    ) -> Result<Vec<f64>, String> {
        if horizon == 0 || training.iter().any(|value| !value.is_finite()) {
            return Err("invalid_window".into());
        }
        let values = match self {
            Self::Naive => naive(training, horizon),
            Self::SeasonalNaive => seasonal_naive(training, horizon, period),
            Self::Drift => drift(training, horizon),
            Self::Ets => ets(training, horizon),
        }?;
        if values.iter().any(|value| !value.is_finite()) {
            return Err("invalid_output".into());
        }
        Ok(values)
    }

    pub(super) fn residuals(self, training: &[f64], period: usize) -> Vec<f64> {
        let minimum = match self {
            Self::Naive => 1,
            Self::SeasonalNaive => period.max(1),
            Self::Drift => 2,
            Self::Ets => 3,
        };
        (minimum..training.len())
            .filter_map(|index| {
                self.forecast(&training[..index], 1, period)
                    .ok()
                    .and_then(|values| values.first().copied())
                    .map(|predicted| (training[index] - predicted).abs())
            })
            .collect()
    }
}

pub(crate) fn seasonal_period(frequency: &str) -> usize {
    let normalized = frequency.trim().to_uppercase();
    match normalized.as_str() {
        "S" => 60,
        "T" | "MIN" => 60,
        "H" => 24,
        "D" => 7,
        "B" => 5,
        "W" => 52,
        "M" => 12,
        "Q" => 4,
        "Y" | "A" => 1,
        _ => 1,
    }
}

fn naive(training: &[f64], horizon: usize) -> Result<Vec<f64>, String> {
    let last = training.last().copied().ok_or("insufficient_history")?;
    Ok(vec![last; horizon])
}

fn seasonal_naive(training: &[f64], horizon: usize, period: usize) -> Result<Vec<f64>, String> {
    if period == 0 || training.len() < period {
        return Err("seasonal_history_too_short".into());
    }
    let start = training.len() - period;
    Ok((0..horizon)
        .map(|step| training[start + step % period])
        .collect())
}

fn drift(training: &[f64], horizon: usize) -> Result<Vec<f64>, String> {
    if training.len() < 2 {
        return Err("insufficient_history".into());
    }
    let first = training[0];
    let last = training[training.len() - 1];
    let slope = (last - first) / (training.len() - 1) as f64;
    Ok((1..=horizon)
        .map(|step| last + slope * step as f64)
        .collect())
}

fn ets(training: &[f64], horizon: usize) -> Result<Vec<f64>, String> {
    if training.len() < 3 {
        return Err("ets_history_too_short".into());
    }
    let candidates = [0.2, 0.5, 0.8];
    let mut best = None;
    for alpha in candidates {
        for beta in candidates {
            let (level, trend, error) = holt_fit(training, alpha, beta);
            if best.is_none_or(|(_, _, best_error)| error < best_error) {
                best = Some((level, trend, error));
            }
        }
    }
    let (level, trend, _) = best.ok_or("ets_unavailable")?;
    Ok((1..=horizon)
        .map(|step| level + trend * step as f64)
        .collect())
}

fn holt_fit(training: &[f64], alpha: f64, beta: f64) -> (f64, f64, f64) {
    let mut level = training[0];
    let mut trend = training[1] - training[0];
    let mut error = 0.0;
    for value in training.iter().skip(1) {
        let predicted = level + trend;
        error += (value - predicted).powi(2);
        let previous_level = level;
        level = alpha * value + (1.0 - alpha) * predicted;
        trend = beta * (level - previous_level) + (1.0 - beta) * trend;
    }
    (level, trend, error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baselines_only_use_training_values() {
        let training = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(
            Baseline::Naive.forecast(&training, 2, 1).unwrap(),
            [4.0, 4.0]
        );
        assert_eq!(
            Baseline::Drift.forecast(&training, 2, 1).unwrap(),
            [5.0, 6.0]
        );
    }

    #[test]
    fn seasonal_naive_repeats_the_last_cycle() {
        let result = Baseline::SeasonalNaive
            .forecast(&[1.0, 2.0, 10.0, 20.0], 3, 2)
            .unwrap();
        assert_eq!(result, [10.0, 20.0, 10.0]);
    }

    #[test]
    fn ets_tracks_a_linear_series() {
        let result = Baseline::Ets
            .forecast(&[1.0, 2.0, 3.0, 4.0, 5.0], 2, 1)
            .unwrap();
        assert!((result[0] - 6.0).abs() < 0.5);
    }
}
