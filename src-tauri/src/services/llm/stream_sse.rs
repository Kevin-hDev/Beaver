use futures_util::{Stream, StreamExt};

const MAX_EVENT_BYTES: usize = crate::services::secure_http::LLM_BODY_LIMIT;
const MAX_STREAM_BYTES: usize = crate::services::secure_http::MAX_AUTHENTICATED_BODY_LIMIT;
const STREAM_LIMIT_ERROR: &str = "provider_connection_failed";

struct SseWireBudget {
    event_bytes: usize,
    total_bytes: usize,
    max_event_bytes: usize,
    max_total_bytes: usize,
    at_line_start: bool,
    last_was_cr: bool,
    blank_line_ended_on_cr: bool,
}

impl SseWireBudget {
    fn new(max_event_bytes: usize, max_total_bytes: usize) -> Self {
        Self {
            event_bytes: 0,
            total_bytes: 0,
            max_event_bytes,
            max_total_bytes,
            at_line_start: true,
            last_was_cr: false,
            blank_line_ended_on_cr: false,
        }
    }

    fn observe(&mut self, bytes: &[u8]) -> Result<(), &'static str> {
        self.total_bytes = self
            .total_bytes
            .checked_add(bytes.len())
            .filter(|total| *total <= self.max_total_bytes)
            .ok_or(STREAM_LIMIT_ERROR)?;

        for byte in bytes {
            self.event_bytes = self
                .event_bytes
                .checked_add(1)
                .filter(|event| *event <= self.max_event_bytes)
                .ok_or(STREAM_LIMIT_ERROR)?;
            self.observe_line_ending(*byte);
        }
        Ok(())
    }

    fn observe_line_ending(&mut self, byte: u8) {
        match byte {
            b'\r' => {
                self.blank_line_ended_on_cr = self.at_line_start;
                if self.blank_line_ended_on_cr {
                    self.event_bytes = 0;
                }
                self.at_line_start = true;
                self.last_was_cr = true;
            }
            b'\n' if self.last_was_cr => {
                if self.blank_line_ended_on_cr {
                    self.event_bytes = 0;
                }
                self.last_was_cr = false;
                self.blank_line_ended_on_cr = false;
            }
            b'\n' => {
                if self.at_line_start {
                    self.event_bytes = 0;
                }
                self.at_line_start = true;
                self.blank_line_ended_on_cr = false;
            }
            _ => {
                self.at_line_start = false;
                self.last_was_cr = false;
                self.blank_line_ended_on_cr = false;
            }
        }
    }
}

fn bounded_wire<S, B, E>(
    stream: S,
    max_event_bytes: usize,
    max_total_bytes: usize,
) -> impl Stream<Item = Result<B, &'static str>>
where
    S: Stream<Item = Result<B, E>>,
    B: AsRef<[u8]>,
{
    let mut budget = SseWireBudget::new(max_event_bytes, max_total_bytes);
    stream.map(move |chunk| {
        let chunk = chunk.map_err(|_| STREAM_LIMIT_ERROR)?;
        budget.observe(chunk.as_ref())?;
        Ok(chunk)
    })
}

pub(crate) fn bounded_response(
    response: reqwest::Response,
) -> impl Stream<Item = Result<bytes::Bytes, &'static str>> {
    // The limit must run before eventsource-stream, which buffers an unfinished event internally.
    bounded_wire(response.bytes_stream(), MAX_EVENT_BYTES, MAX_STREAM_BYTES)
}

pub(crate) fn is_done_marker(data: &str) -> bool {
    data.trim() == "[DONE]"
}

pub(crate) fn parse_json(data: &str) -> Result<serde_json::Value, String> {
    if data.len() > MAX_EVENT_BYTES {
        return Err("provider_connection_failed".to_string());
    }
    serde_json::from_str(data).map_err(|_| "provider_connection_failed".to_string())
}

#[cfg(test)]
mod tests {
    use super::{is_done_marker, parse_json, MAX_EVENT_BYTES};
    use bytes::Bytes;
    use futures_util::{stream, StreamExt};

    #[test]
    fn recognizes_done_marker_with_whitespace() {
        assert!(is_done_marker(" [DONE]\n"));
    }

    #[test]
    fn rejects_regular_json_chunk() {
        assert!(!is_done_marker(r#"{"choices":[]}"#));
    }

    #[test]
    fn rejects_an_oversized_external_event_before_json_allocation() {
        assert!(parse_json(&" ".repeat(MAX_EVENT_BYTES + 1)).is_err());
    }

    #[tokio::test]
    async fn rejects_a_fragmented_event_before_the_sse_parser_can_buffer_it() {
        let source = stream::iter([
            Ok::<_, ()>(Bytes::from_static(b"data: 12")),
            Ok(Bytes::from_static(b"345")),
        ]);
        let mut bounded = super::bounded_wire(source, 10, 100);

        assert!(bounded.next().await.unwrap().is_ok());
        assert!(bounded.next().await.unwrap().is_err());
    }

    #[tokio::test]
    async fn a_complete_event_resets_only_the_per_event_budget() {
        let source = stream::iter([
            Ok::<_, ()>(Bytes::from_static(b"data:a\n\n")),
            Ok(Bytes::from_static(b"data:b\n\n")),
        ]);
        let mut bounded = super::bounded_wire(source, 8, 32);

        assert!(bounded.next().await.unwrap().is_ok());
        assert!(bounded.next().await.unwrap().is_ok());
    }

    #[tokio::test]
    async fn rejects_many_valid_events_when_the_total_stream_budget_is_exhausted() {
        let source = stream::iter([
            Ok::<_, ()>(Bytes::from_static(b"data:a\n\n")),
            Ok(Bytes::from_static(b"data:b\n\n")),
        ]);
        let mut bounded = super::bounded_wire(source, 8, 12);

        assert!(bounded.next().await.unwrap().is_ok());
        assert!(bounded.next().await.unwrap().is_err());
    }
}
