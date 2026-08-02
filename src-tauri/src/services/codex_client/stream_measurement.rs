use crate::services::agent_local::stream_buffer::StreamEventSink;
use crate::services::agent_local::types_ollama::StreamOutcome;
use crate::services::provider_usage::RequestMeasurement;

use super::stream_accumulator::StreamAccumulator;

pub(super) struct StreamMeasurement<'a> {
    inner: Option<&'a mut RequestMeasurement>,
}

impl<'a> StreamMeasurement<'a> {
    pub(super) fn new(inner: Option<&'a mut RequestMeasurement>) -> Self {
        Self { inner }
    }

    pub(super) fn mark_headers(&mut self) {
        if let Some(measurement) = self.inner.as_mut() {
            measurement.mark_headers();
        }
    }

    pub(super) fn mark_first_event(&mut self) {
        if let Some(measurement) = self.inner.as_mut() {
            measurement.mark_first_event();
        }
    }

    pub(super) fn mark_first_useful(&mut self) {
        if let Some(measurement) = self.inner.as_mut() {
            measurement.mark_first_useful();
        }
    }

    pub(super) fn apply(
        &mut self,
        accumulator: &mut StreamAccumulator<'_>,
        on_event: &impl StreamEventSink,
        event: &serde_json::Value,
    ) -> Result<Option<StreamOutcome>, String> {
        self.mark_first_event();
        let useful_before = accumulator.has_useful_output();
        let outcome = accumulator.apply(on_event, event)?;
        if !useful_before && accumulator.has_useful_output() {
            self.mark_first_useful();
        }
        Ok(outcome)
    }
}
