use crate::services::agent_local::stream_buffer::StreamEventSink;
use crate::services::agent_local::types_ollama::StreamOutcome;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

use super::{
    stream_accumulator::StreamAccumulator, stream_measurement::StreamMeasurement, stream_protocol,
};

pub(super) async fn consume_sse_with_accumulator(
    on_event: &impl StreamEventSink,
    resp: reqwest::Response,
    cancel: CancellationToken,
    idle_timeout: std::time::Duration,
    mut accumulator: StreamAccumulator<'_>,
    measurement: &mut StreamMeasurement<'_>,
) -> Result<StreamOutcome, String> {
    let sse = crate::services::llm::stream_sse::bounded_response(resp).eventsource();
    futures_util::pin_mut!(sse);
    loop {
        let event = tokio::select! {
            biased;
            _ = cancel.cancelled() => return Err("Annulé".to_string()),
            _ = tokio::time::sleep(idle_timeout) => {
                return Err("provider_temporarily_unavailable".to_string());
            }
            ev = sse.next() => match ev {
                Some(Ok(e)) => e,
                Some(Err(_)) => return Err("provider_connection_failed".to_string()),
                None => return Err(stream_protocol::closed_before_completed()),
            },
        };

        if event.data.trim() == "[DONE]" {
            break;
        }
        let parsed = crate::services::llm::stream_sse::parse_json(&event.data)?;
        if let Some(outcome) = measurement.apply(&mut accumulator, on_event, &parsed)? {
            return Ok(outcome);
        }
    }
    Err(stream_protocol::closed_before_completed())
}
