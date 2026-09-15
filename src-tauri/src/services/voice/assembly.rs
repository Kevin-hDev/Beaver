use crate::services::voice::{
    errors::VoiceError,
    limits::{MAX_TRANSCRIPT_CHARS, VOICE_SAMPLE_RATE},
    model::recognizer::RecognitionSlice,
    slicing::{AssemblyPlan, SliceInterval},
};
use zeroize::Zeroize;

pub struct SliceResult {
    pub interval: SliceInterval,
    pub recognition: RecognitionSlice,
}

#[derive(Default)]
pub struct AssemblyAccumulator {
    text: String,
    chars: usize,
    next_interval: usize,
}

impl AssemblyAccumulator {
    pub fn push(&mut self, plan: &AssemblyPlan, slice: &SliceResult) -> Result<(), VoiceError> {
        let expected = plan
            .intervals()
            .get(self.next_interval)
            .filter(|expected| **expected == slice.interval)
            .ok_or_else(VoiceError::configuration_unavailable)?;
        append_central_tokens(
            &mut self.text,
            &mut self.chars,
            plan,
            *expected,
            &slice.recognition,
        )?;
        self.next_interval += 1;
        Ok(())
    }

    pub fn finish(mut self, plan: &AssemblyPlan) -> Result<String, VoiceError> {
        if self.next_interval == plan.intervals().len() {
            Ok(std::mem::take(&mut self.text))
        } else {
            Err(VoiceError::configuration_unavailable())
        }
    }
}

impl Drop for AssemblyAccumulator {
    fn drop(&mut self) {
        self.text.zeroize();
    }
}

#[cfg(test)]
pub fn assemble(plan: &AssemblyPlan, slices: &[SliceResult]) -> Result<String, VoiceError> {
    if slices.len() != plan.intervals().len() {
        return Err(VoiceError::configuration_unavailable());
    }
    let mut accumulator = AssemblyAccumulator::default();
    for slice in slices {
        accumulator.push(plan, slice)?;
    }
    accumulator.finish(plan)
}

fn append_central_tokens(
    text: &mut String,
    chars: &mut usize,
    plan: &AssemblyPlan,
    interval: SliceInterval,
    recognition: &RecognitionSlice,
) -> Result<(), VoiceError> {
    let timestamps = recognition
        .timestamps_ms
        .as_ref()
        .filter(|values| values.len() == recognition.tokens.len())
        .ok_or_else(VoiceError::configuration_unavailable)?;
    let durations = recognition.durations_ms.as_ref();
    for (index, token) in recognition.tokens.iter().enumerate() {
        let midpoint_ms = timestamps[index]
            .saturating_add(durations.and_then(|v| v.get(index)).copied().unwrap_or(0) / 2);
        let absolute = interval.context_start.saturating_add(
            usize::try_from(midpoint_ms)
                .unwrap_or(usize::MAX)
                .saturating_mul(VOICE_SAMPLE_RATE as usize)
                / 1_000,
        );
        let in_central = absolute >= interval.central_start
            && (absolute < interval.central_end
                || (plan.is_last(interval) && absolute == interval.central_end));
        if in_central {
            let token_chars = token.chars().count();
            if chars.saturating_add(token_chars) > MAX_TRANSCRIPT_CHARS {
                return Err(VoiceError::configuration_unavailable());
            }
            text.push_str(token);
            *chars = chars.saturating_add(token_chars);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(interval: SliceInterval, tokens: &[&str], times: &[u64]) -> SliceResult {
        SliceResult {
            interval,
            recognition: RecognitionSlice {
                text: tokens.concat(),
                tokens: tokens.iter().map(|token| (*token).into()).collect(),
                timestamps_ms: Some(times.to_vec()),
                durations_ms: Some(vec![0; tokens.len()]),
            },
        }
    }

    #[test]
    fn assigns_overlap_by_timing_without_text_deduplication() {
        let plan = AssemblyPlan::new(32 * 16_000, 16_000).unwrap();
        let first = plan.intervals()[0];
        let second = plan.intervals()[1];
        let slices = [
            result(first, &["oui", " oui", "."], &[29_000, 30_100, 29_900]),
            result(second, &[" oui", "再见"], &[100, 1_500]),
        ];
        assert_eq!(assemble(&plan, &slices).unwrap(), "oui.再见");
    }

    #[test]
    fn refuses_blind_assembly_without_timestamps() {
        let plan = AssemblyPlan::new(1, 0).unwrap();
        let mut slice = result(plan.intervals()[0], &["x"], &[0]);
        slice.recognition.timestamps_ms = None;
        assert!(assemble(&plan, &[slice]).is_err());
    }
}
