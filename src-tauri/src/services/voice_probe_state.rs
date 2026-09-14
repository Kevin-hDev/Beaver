use serde::Serialize;
use std::collections::VecDeque;

pub(super) const MAX_LEVEL_HISTORY: usize = 100;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VoiceProbeStatus {
    #[default]
    Idle,
    Starting,
    Listening,
    Stopping,
    Error,
}

#[derive(Clone, Debug, Serialize)]
pub struct VoiceProbeSnapshot {
    pub status: VoiceProbeStatus,
    pub sample_count: u64,
    pub levels: Vec<f32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VoiceProbeError {
    Busy,
    Unavailable,
}

impl VoiceProbeError {
    pub fn public_code(self) -> &'static str {
        match self {
            Self::Busy => "voice-probe-busy",
            Self::Unavailable => "voice-probe-unavailable",
        }
    }
}

#[derive(Default)]
pub(super) struct ProbeState {
    status: VoiceProbeStatus,
    sample_count: u64,
    levels: VecDeque<f32>,
}

impl ProbeState {
    pub(super) fn begin(&mut self) -> Result<(), VoiceProbeError> {
        if !matches!(
            self.status,
            VoiceProbeStatus::Idle | VoiceProbeStatus::Error
        ) {
            return Err(VoiceProbeError::Busy);
        }
        self.status = VoiceProbeStatus::Starting;
        self.sample_count = 0;
        self.levels.clear();
        Ok(())
    }

    pub(super) fn mark_listening(&mut self) {
        self.status = VoiceProbeStatus::Listening;
    }

    pub(super) fn mark_stopping(&mut self) {
        self.status = VoiceProbeStatus::Stopping;
    }

    pub(super) fn fail(&mut self) {
        self.status = VoiceProbeStatus::Error;
    }

    pub(super) fn finish(&mut self) {
        self.status = VoiceProbeStatus::Idle;
    }

    pub(super) fn record(&mut self, sample_count: u64, level: f32) {
        if self.levels.len() == MAX_LEVEL_HISTORY {
            self.levels.pop_front();
        }
        self.sample_count = sample_count;
        self.levels.push_back(level.clamp(0.0, 1.0));
    }

    pub(super) fn snapshot(&self) -> VoiceProbeSnapshot {
        VoiceProbeSnapshot {
            status: self.status,
            sample_count: self.sample_count,
            levels: self.levels.iter().copied().collect(),
        }
    }
}
