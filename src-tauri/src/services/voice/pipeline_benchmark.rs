use super::{
    audio_buffer::AudioBuffer,
    limits::VOICE_SAMPLE_RATE,
    model::{benchmarks::Benchmark, lifecycle::ModelLease},
};

pub fn candidate(lease: &ModelLease, audio: &AudioBuffer) -> Benchmark {
    Benchmark {
        model_id: lease.prepared().entry_id.clone(),
        revision: lease.prepared().revision.clone(),
        audio_ms: u64::try_from(audio.samples().len())
            .unwrap_or(u64::MAX)
            .saturating_mul(1_000)
            / u64::from(VOICE_SAMPLE_RATE),
        compute_ms: 0,
        profile: lease.prepared().profile.clone(),
    }
}
