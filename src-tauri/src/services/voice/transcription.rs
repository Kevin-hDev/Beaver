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
    pub lost_samples: u64,
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
    for interval in plan.intervals() {
        let mut samples: Vec<f32> = audio.samples()[interval.context_start..interval.context_end]
            .iter()
            .map(|sample| f32::from(*sample) / i16::MAX as f32)
            .collect();
        let recognition = recognizer::recognize_slice(lease.prepared_mut(), &samples, language);
        samples.zeroize();
        let result = SliceResult {
            interval: *interval,
            recognition: recognition?,
        };
        assembly.push(&plan, &result)?;
    }
    let text = assembly.finish(&plan)?;
    let lost_samples = audio.lost_samples();
    Ok(TranscriptionResult { text, lost_samples })
}
