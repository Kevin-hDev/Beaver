use zeroize::Zeroize;

use crate::services::voice::{
    assembly::{AssemblyAccumulator, SliceResult},
    audio_buffer::AudioBuffer,
    errors::VoiceError,
    model::{lifecycle::ModelLease, recognizer},
    slicing::AssemblyPlan,
};

pub struct TranscriptionResult {
    pub text: String,
}

pub fn transcribe(
    lease: &mut ModelLease,
    audio: &mut AudioBuffer,
    language: &recognizer::EffectiveLanguage,
) -> Result<TranscriptionResult, VoiceError> {
    let result = transcribe_inner(lease, audio, language);
    audio.clear();
    result
}

fn transcribe_inner(
    lease: &mut ModelLease,
    audio: &AudioBuffer,
    language: &recognizer::EffectiveLanguage,
) -> Result<TranscriptionResult, VoiceError> {
    let plan = AssemblyPlan::new(audio.samples().len(), 16_000)
        .ok_or_else(VoiceError::invalid_settings)?;
    let mut assembly = AssemblyAccumulator::default();
    for (index, interval) in plan.intervals().iter().enumerate() {
        let mut recognition = recognize_interval(lease, audio, *interval, language)?;
        if index == 0 && recognition.timestamps_ms.is_none() {
            if plan.intervals().len() == 1 {
                return Ok(TranscriptionResult {
                    text: recognition.take_text(),
                });
            }
            return transcribe_without_timestamps(lease, audio, language);
        }
        let result = SliceResult {
            interval: *interval,
            recognition,
        };
        assembly.push(&plan, &result)?;
    }
    let text = assembly.finish(&plan)?;
    Ok(TranscriptionResult { text })
}

fn transcribe_without_timestamps(
    lease: &mut ModelLease,
    audio: &AudioBuffer,
    language: &recognizer::EffectiveLanguage,
) -> Result<TranscriptionResult, VoiceError> {
    let plan = AssemblyPlan::without_overlap_at_quiet_points(audio.samples())
        .ok_or_else(VoiceError::invalid_settings)?;
    let mut text = zeroize::Zeroizing::new(String::new());
    for interval in plan.intervals() {
        let recognition = recognize_interval(lease, audio, *interval, language)?;
        append_without_overlap(&mut text, &recognition.text)?;
    }
    Ok(TranscriptionResult {
        text: std::mem::take(&mut *text),
    })
}

fn recognize_interval(
    lease: &mut ModelLease,
    audio: &AudioBuffer,
    interval: crate::services::voice::slicing::SliceInterval,
    language: &recognizer::EffectiveLanguage,
) -> Result<recognizer::RecognitionSlice, VoiceError> {
    let mut samples: Vec<f32> = audio.samples()[interval.context_start..interval.context_end]
        .iter()
        .map(|sample| f32::from(*sample) / i16::MAX as f32)
        .collect();
    let recognition = recognizer::recognize_slice(lease.prepared_mut(), &samples, language);
    samples.zeroize();
    recognition
}

fn append_without_overlap(text: &mut String, next: &str) -> Result<(), VoiceError> {
    let separator = text
        .chars()
        .next_back()
        .zip(next.chars().next())
        .is_some_and(|(left, right)| left.is_ascii_alphanumeric() && right.is_ascii_alphanumeric());
    let added = next.chars().count() + usize::from(separator);
    if text.chars().count().saturating_add(added)
        > crate::services::voice::limits::MAX_TRANSCRIPT_CHARS
    {
        return Err(VoiceError::configuration_unavailable());
    }
    if separator {
        text.push(' ');
    }
    text.push_str(next);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn fallback_separator_preserves_words_and_unspaced_scripts() {
        let mut latin = "hello".to_owned();
        super::append_without_overlap(&mut latin, "world").unwrap();
        assert_eq!(latin, "hello world");
        let mut han = "你".to_owned();
        super::append_without_overlap(&mut han, "好").unwrap();
        assert_eq!(han, "你好");
    }
}
