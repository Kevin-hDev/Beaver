use std::io;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures_util::{stream::BoxStream, StreamExt};
use reqwest::Response;
use serde_json::Value;
use sse_stream::{Error as SseError, Sse, SseStream};

use crate::services::secure_http::{read_bounded, SecureHttpError, MCP_BODY_LIMIT};

pub(super) async fn json(
    response: Response,
    remaining: &AtomicUsize,
) -> Result<zeroize::Zeroizing<Vec<u8>>, SecureHttpError> {
    let limit = remaining.load(Ordering::Acquire).min(MCP_BODY_LIMIT);
    if limit == 0 {
        return Err(SecureHttpError::BodyTooLarge);
    }
    let body = read_bounded(response, limit).await?;
    spend(remaining, body.len()).map_err(|_| SecureHttpError::BodyTooLarge)?;
    Ok(body)
}

pub(super) fn sse(
    response: Response,
    requested_event_limit: usize,
    remaining: Arc<AtomicUsize>,
) -> BoxStream<'static, Result<Sse, SseError>> {
    let mut limiter = EventLimiter::new(requested_event_limit.min(MCP_BODY_LIMIT));
    let stream = response.bytes_stream().map(move |chunk| {
        let bytes = chunk.map_err(io::Error::other)?;
        spend(&remaining, bytes.len())?;
        limiter.observe(&bytes)?;
        Ok::<_, io::Error>(bytes)
    });
    SseStream::from_bytes_stream(stream)
        .map(|event| {
            let event = event?;
            let mut event = event;
            if let Some(data) = event.data.as_deref() {
                if let Ok(mut value) = serde_json::from_str::<Value>(data) {
                    let changed = normalize_cache_metadata(&mut value)
                        .map_err(|error| SseError::Body(Box::new(error)))?;
                    if changed {
                        event.data = Some(
                            serde_json::to_string(&value)
                                .map_err(|error| SseError::Body(Box::new(error)))?,
                        );
                    }
                }
            }
            Ok(event)
        })
        .boxed()
}

pub(super) fn normalize_cache_metadata(value: &mut Value) -> Result<bool, io::Error> {
    if value
        .get("result")
        .and_then(|result| result.get("ttlMs"))
        .is_some_and(|ttl| ttl.as_u64().is_none())
    {
        return Err(io::Error::other("durée de cache MCP invalide"));
    }
    let Some(result) = value.get_mut("result").and_then(Value::as_object_mut) else {
        return Ok(false);
    };
    if result
        .get("cacheScope")
        .is_some_and(|scope| scope != "public" && scope != "private")
    {
        result.remove("cacheScope");
        return Ok(true);
    }
    Ok(false)
}

fn spend(remaining: &AtomicUsize, count: usize) -> Result<(), io::Error> {
    remaining
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |left| {
            left.checked_sub(count)
        })
        .map(|_| ())
        .map_err(|_| io::Error::other("réponse MCP trop volumineuse"))
}

struct EventLimiter {
    max: usize,
    used: usize,
    line_has_content: bool,
    previous_cr: bool,
}

impl EventLimiter {
    fn new(max: usize) -> Self {
        Self {
            max,
            used: 0,
            line_has_content: false,
            previous_cr: false,
        }
    }

    fn observe(&mut self, chunk: &[u8]) -> Result<(), io::Error> {
        for &byte in chunk {
            if self.previous_cr {
                self.previous_cr = false;
                if byte == b'\n' {
                    continue;
                }
            }
            self.used = self
                .used
                .checked_add(1)
                .ok_or_else(|| io::Error::other("réponse MCP trop volumineuse"))?;
            match byte {
                b'\r' => {
                    self.finish_line();
                    self.previous_cr = true;
                }
                b'\n' => self.finish_line(),
                _ => self.line_has_content = true,
            }
            if self.used > self.max {
                return Err(io::Error::other("événement MCP trop volumineux"));
            }
        }
        Ok(())
    }

    fn finish_line(&mut self) {
        if !self.line_has_content {
            self.used = 0;
        }
        self.line_has_content = false;
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_cache_metadata, EventLimiter};
    use serde_json::json;

    #[test]
    fn fragmented_events_and_separatorless_stream_are_bounded() {
        let mut limiter = EventLimiter::new(14);
        limiter.observe(b"data: a\r").unwrap();
        limiter.observe(b"\n\r\n").unwrap();
        limiter.observe(b"data: b\n\n").unwrap();
        assert!(limiter.observe(b"data: long-without-separator").is_err());
    }

    #[test]
    fn malformed_cache_ttl_is_rejected_before_sdk_deserialization() {
        for ttl in [json!(-1), json!("1000")] {
            assert!(normalize_cache_metadata(&mut json!({"result": {"ttlMs": ttl}})).is_err());
        }
        assert!(normalize_cache_metadata(&mut json!({"result": {"ttlMs": 0}})).is_ok());
    }
}
