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
}
