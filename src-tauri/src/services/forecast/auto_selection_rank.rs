use std::cmp::Ordering;

use super::{AutoCandidate, CandidateBacktest, ResourceFit};

pub(super) fn compare(left: &AutoCandidate, right: &AutoCandidate, rolling: bool) -> Ordering {
    if rolling {
        match (&left.backtest, &right.backtest) {
            (Some(left), Some(right)) => return compare_backtests(left, right),
            (Some(_), None) => return Ordering::Less,
            (None, Some(_)) => return Ordering::Greater,
            (None, None) => {}
        }
    }
    resource_rank(right.resource_fit)
        .cmp(&resource_rank(left.resource_fit))
        .then_with(|| left.estimated_ram_mb.cmp(&right.estimated_ram_mb))
        .then_with(|| left.model_id.cmp(&right.model_id))
}

fn compare_backtests(left: &CandidateBacktest, right: &CandidateBacktest) -> Ordering {
    baseline_rank(right.beats_best_baseline)
        .cmp(&baseline_rank(left.beats_best_baseline))
        .then_with(|| {
            crate::services::forecast::evaluation::ranking::compare_quality(
                &left.metrics,
                left.calibration.as_ref(),
                left.duration_ms,
                left.max_memory_mb,
                &right.metrics,
                right.calibration.as_ref(),
                right.duration_ms,
                right.max_memory_mb,
            )
        })
}

fn baseline_rank(value: Option<bool>) -> u8 {
    match value {
        Some(true) => 2,
        None => 1,
        Some(false) => 0,
    }
}

fn resource_rank(fit: ResourceFit) -> u8 {
    match fit {
        ResourceFit::Comfortable => 4,
        ResourceFit::Cloud => 3,
        ResourceFit::Constrained => 2,
        ResourceFit::Unknown => 1,
        ResourceFit::Insufficient => 0,
    }
}
