pub fn outlier_count(values: &[f64]) -> usize {
    if values.len() < 8 {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let q1 = percentile(&sorted, 0.25);
    let q3 = percentile(&sorted, 0.75);
    let iqr = q3 - q1;
    if iqr <= f64::EPSILON {
        return 0;
    }
    let lower = q1 - 3.0 * iqr;
    let upper = q3 + 3.0 * iqr;
    values
        .iter()
        .filter(|value| **value < lower || **value > upper)
        .count()
}

pub fn has_regime_shift(values: &[f64]) -> bool {
    if values.len() < 12 {
        return false;
    }
    let mid = values.len() / 2;
    let first_mean = mean(&values[..mid]);
    let second_mean = mean(&values[mid..]);
    let deviation = standard_deviation(values);
    deviation > f64::EPSILON && (second_mean - first_mean).abs() > 2.5 * deviation
}

fn percentile(sorted: &[f64], level: f64) -> f64 {
    let index = ((sorted.len() - 1) as f64 * level).round() as usize;
    sorted[index]
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn standard_deviation(values: &[f64]) -> f64 {
    let average = mean(values);
    let variance = values
        .iter()
        .map(|value| (value - average).powi(2))
        .sum::<f64>()
        / values.len() as f64;
    variance.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_large_isolated_outlier() {
        assert_eq!(outlier_count(&[1.0, 1.1, 0.9, 1.0, 1.2, 0.8, 1.0, 50.0]), 1);
    }
}
