use super::stats;
use super::variable_samples::Sample;
use super::VariableImportance;

pub(super) fn importance(
    name: &str,
    training: &[Sample],
    validation: &[Sample],
) -> VariableImportance {
    let x_mean = stats::mean(
        &training
            .iter()
            .map(|sample| sample.value)
            .collect::<Vec<_>>(),
    );
    let y_mean = stats::mean(
        &training
            .iter()
            .map(|sample| sample.delta)
            .collect::<Vec<_>>(),
    );
    let numerator: f64 = training
        .iter()
        .map(|sample| (sample.value - x_mean) * (sample.delta - y_mean))
        .sum();
    let denominator: f64 = training
        .iter()
        .map(|sample| (sample.value - x_mean).powi(2))
        .sum();
    let coefficient = if denominator <= f64::EPSILON || !numerator.is_finite() {
        0.0
    } else {
        finite_or_zero(numerator / denominator)
    };
    let intercept = finite_or_zero(y_mean - coefficient * x_mean);
    let validation_mae = mae(
        validation,
        |index| validation[index].value,
        intercept,
        coefficient,
    );
    let rotations = [1, validation.len() / 3, validation.len() * 2 / 3];
    let permuted_mae = finite_or_zero(
        rotations
            .iter()
            .map(|offset| {
                mae(
                    validation,
                    |index| validation[(index + offset) % validation.len()].value,
                    intercept,
                    coefficient,
                )
            })
            .sum::<f64>()
            / rotations.len() as f64,
    );
    let score = finite_or_zero(permuted_mae - validation_mae).max(0.0);
    VariableImportance {
        name: name.to_string(),
        score,
        normalized_score: 0.0,
        direction: direction(coefficient).into(),
        validation_mae,
        permuted_mae,
    }
}

fn mae<F: Fn(usize) -> f64>(samples: &[Sample], value: F, intercept: f64, coefficient: f64) -> f64 {
    finite_or_zero(
        samples
            .iter()
            .enumerate()
            .map(|(index, sample)| (sample.delta - intercept - coefficient * value(index)).abs())
            .sum::<f64>()
            / samples.len().max(1) as f64,
    )
}

fn finite_or_zero(value: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

fn direction(coefficient: f64) -> &'static str {
    if coefficient > f64::EPSILON {
        "positive"
    } else if coefficient < -f64::EPSILON {
        "negative"
    } else {
        "neutral"
    }
}
