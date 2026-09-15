use crate::services::voice::limits::{MAX_ASR_OVERLAP_SAMPLES, MAX_ASR_SLICE_SAMPLES};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SliceInterval {
    pub context_start: usize,
    pub context_end: usize,
    pub central_start: usize,
    pub central_end: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssemblyPlan {
    total_samples: usize,
    intervals: Vec<SliceInterval>,
}

impl AssemblyPlan {
    pub fn new(total_samples: usize, overlap_samples: usize) -> Option<Self> {
        if total_samples == 0 || overlap_samples > MAX_ASR_OVERLAP_SAMPLES {
            return None;
        }
        let mut intervals = Vec::with_capacity(total_samples.div_ceil(MAX_ASR_SLICE_SAMPLES));
        let mut central_start = 0;
        while central_start < total_samples {
            let central_end = (central_start + MAX_ASR_SLICE_SAMPLES).min(total_samples);
            intervals.push(SliceInterval {
                context_start: central_start.saturating_sub(overlap_samples),
                context_end: central_end
                    .saturating_add(overlap_samples)
                    .min(total_samples),
                central_start,
                central_end,
            });
            central_start = central_end;
        }
        Some(Self {
            total_samples,
            intervals,
        })
    }

    pub fn intervals(&self) -> &[SliceInterval] {
        &self.intervals
    }

    pub fn is_last(&self, interval: SliceInterval) -> bool {
        interval.central_end == self.total_samples
    }

    pub fn without_overlap_at_quiet_points(samples: &[i16]) -> Option<Self> {
        if samples.is_empty() {
            return None;
        }
        let mut intervals = Vec::with_capacity(samples.len().div_ceil(MAX_ASR_SLICE_SAMPLES));
        let mut start = 0;
        while start < samples.len() {
            let target = start
                .saturating_add(MAX_ASR_SLICE_SAMPLES)
                .min(samples.len());
            let end = if target == samples.len() {
                target
            } else {
                quiet_boundary(samples, start, target)
            };
            intervals.push(SliceInterval {
                context_start: start,
                context_end: end,
                central_start: start,
                central_end: end,
            });
            start = end;
        }
        Some(Self {
            total_samples: samples.len(),
            intervals,
        })
    }
}

fn quiet_boundary(samples: &[i16], start: usize, target: usize) -> usize {
    const SEARCH_SAMPLES: usize = 16_000;
    const FRAME_SAMPLES: usize = 320;

    let search_start = target
        .saturating_sub(SEARCH_SAMPLES)
        .max(start.saturating_add(FRAME_SAMPLES));
    (search_start..target)
        .step_by(FRAME_SAMPLES)
        .filter_map(|frame_start| {
            let frame_end = frame_start.checked_add(FRAME_SAMPLES)?.min(target);
            let energy = samples[frame_start..frame_end]
                .iter()
                .fold(0_u64, |sum, sample| {
                    sum.saturating_add(u64::from(sample.unsigned_abs()))
                });
            Some((energy, frame_end))
        })
        .min_by_key(|(energy, _)| *energy)
        .map(|(_, end)| end)
        .unwrap_or(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_overlap_without_copying_all_slices() {
        let plan = AssemblyPlan::new(MAX_ASR_SLICE_SAMPLES * 2 + 1, 16_000).unwrap();
        assert_eq!(plan.intervals().len(), 3);
        assert_eq!(
            plan.intervals()[0].context_end,
            MAX_ASR_SLICE_SAMPLES + 16_000
        );
        assert_eq!(
            plan.intervals()[1].context_start,
            MAX_ASR_SLICE_SAMPLES - 16_000
        );
        assert!(AssemblyPlan::new(1, MAX_ASR_OVERLAP_SAMPLES + 1).is_none());
    }

    #[test]
    fn timestamp_free_plan_uses_a_quiet_boundary_without_overlap() {
        let mut samples = vec![10_i16; MAX_ASR_SLICE_SAMPLES + 1];
        samples[MAX_ASR_SLICE_SAMPLES - 8_000..MAX_ASR_SLICE_SAMPLES - 7_680].fill(0);
        let plan = AssemblyPlan::without_overlap_at_quiet_points(&samples).unwrap();
        assert_eq!(
            plan.intervals()[0].central_end,
            MAX_ASR_SLICE_SAMPLES - 7_680
        );
        assert_eq!(
            plan.intervals()[1].context_start,
            plan.intervals()[0].context_end
        );
    }
}
