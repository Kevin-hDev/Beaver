pub(super) fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len().max(1) as f64
}

pub(super) fn variance(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let center = mean(values);
    values
        .iter()
        .map(|value| (value - center).powi(2))
        .sum::<f64>()
        / values.len() as f64
}

pub(super) fn median(values: &[f64]) -> f64 {
    let mut sorted: Vec<_> = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect();
    if sorted.is_empty() {
        return 0.0;
    }
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[middle - 1] + sorted[middle]) / 2.0
    } else {
        sorted[middle]
    }
}

pub(super) fn mad(values: &[f64], center: f64) -> f64 {
    median(
        &values
            .iter()
            .map(|value| (value - center).abs())
            .collect::<Vec<_>>(),
    )
}

pub(super) fn slope(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let x_mean = (values.len() - 1) as f64 / 2.0;
    let y_mean = mean(values);
    let (numerator, denominator) =
        values
            .iter()
            .enumerate()
            .fold((0.0, 0.0), |(num, den), (index, value)| {
                let x = index as f64 - x_mean;
                (num + x * (value - y_mean), den + x * x)
            });
    if denominator <= f64::EPSILON {
        0.0
    } else {
        numerator / denominator
    }
}

pub(super) fn ks_distance(left: &[f64], right: &[f64]) -> f64 {
    let mut left = left.to_vec();
    let mut right = right.to_vec();
    left.sort_by(f64::total_cmp);
    right.sort_by(f64::total_cmp);
    let mut i = 0usize;
    let mut j = 0usize;
    let mut distance: f64 = 0.0;
    while i < left.len() || j < right.len() {
        let next = match (left.get(i), right.get(j)) {
            (Some(a), Some(b)) => a.min(*b),
            (Some(a), None) => *a,
            (None, Some(b)) => *b,
            (None, None) => break,
        };
        while i < left.len() && left[i] <= next {
            i += 1;
        }
        while j < right.len() && right[j] <= next {
            j += 1;
        }
        distance = distance.max(
            (i as f64 / left.len().max(1) as f64 - j as f64 / right.len().max(1) as f64).abs(),
        );
    }
    distance
}
