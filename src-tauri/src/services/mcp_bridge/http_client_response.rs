use std::borrow::Cow;
use std::sync::{atomic::AtomicUsize, Arc};

use futures_util::stream::BoxStream;
use reqwest::{header, Response, StatusCode};
use rmcp::model::{ErrorCode, ServerJsonRpcMessage};
use rmcp::transport::streamable_http_client::{
    AuthRequiredError, SseError, StreamableHttpError, StreamableHttpPostResponse,
};
use sse_stream::Sse;

use crate::services::secure_http::SecureHttpError;

type WireError = StreamableHttpError<SecureHttpError>;

fn unexpected() -> WireError {
    WireError::UnexpectedServerResponse(Cow::Borrowed("réponse MCP invalide"))
}

fn authentication(response: &Response) -> WireError {
    let challenge = response
        .headers()
        .get(header::WWW_AUTHENTICATE)
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.len() <= 4096)
        .unwrap_or_default();
    WireError::AuthRequired(AuthRequiredError::new(challenge.to_owned()))
}

fn session_id(response: &Response) -> Option<String> {
    response
        .headers()
        .get("mcp-session-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| {
            value.len() <= 256 && value.bytes().all(|byte| (0x20..0x7f).contains(&byte))
        })
        .map(str::to_owned)
}

fn content_type(response: &Response, expected: &str) -> bool {
    response
        .headers()
        .get(header::CONTENT_TYPE)
        .is_some_and(|value| value.as_bytes().starts_with(expected.as_bytes()))
}

pub(super) async fn post(
    response: Response,
    had_session: bool,
    is_request: bool,
    is_discovery: bool,
    event_limit: usize,
    remaining: Arc<AtomicUsize>,
) -> Result<StreamableHttpPostResponse, WireError> {
    let status = response.status();
    if status == StatusCode::UNAUTHORIZED {
        return Err(authentication(&response));
    }
    if status == StatusCode::NOT_FOUND && had_session {
        return Err(WireError::SessionExpired);
    }
    if status == StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
        || status == StatusCode::FORBIDDEN
    {
        return Err(unexpected());
    }
    let session = session_id(&response);
    if status == StatusCode::ACCEPTED || status == StatusCode::NO_CONTENT {
        return Ok(StreamableHttpPostResponse::Accepted);
    }
    if status.is_success() && !is_request && response.content_length() == Some(0) {
        return Ok(StreamableHttpPostResponse::Accepted);
    }
    if content_type(&response, "application/json") {
        let body = super::http_limits::json(response, &remaining)
            .await
            .map_err(WireError::Client)?;
        let mut raw: serde_json::Value = serde_json::from_slice(&body).map_err(|_| unexpected())?;
        super::http_limits::normalize_cache_metadata(&mut raw).map_err(|_| unexpected())?;
        let message: ServerJsonRpcMessage =
            serde_json::from_value(raw).map_err(|_| unexpected())?;
        if is_discovery
            && matches!(&message, ServerJsonRpcMessage::Error(error)
                if !matches!(error.error.code,
                    ErrorCode::METHOD_NOT_FOUND
                        | ErrorCode::UNSUPPORTED_PROTOCOL_VERSION
                        | ErrorCode::MISSING_REQUIRED_CLIENT_CAPABILITY
                        | ErrorCode::HEADER_MISMATCH))
        {
            return Err(unexpected());
        }
        if !status.is_success() && !matches!(message, ServerJsonRpcMessage::Error(_)) {
            return Err(unexpected());
        }
        return Ok(StreamableHttpPostResponse::Json(message, session));
    }
    if status.is_success() && content_type(&response, "text/event-stream") {
        return Ok(StreamableHttpPostResponse::Sse(
            super::http_limits::sse(response, event_limit, remaining),
            session,
        ));
    }
    Err(unexpected())
}

pub(super) fn get(
    response: Response,
    event_limit: usize,
    remaining: Arc<AtomicUsize>,
) -> Result<BoxStream<'static, Result<Sse, SseError>>, WireError> {
    if response.status() == StatusCode::METHOD_NOT_ALLOWED {
        return Err(WireError::ServerDoesNotSupportSse);
    }
    if response.status() == StatusCode::UNAUTHORIZED {
        return Err(authentication(&response));
    }
    if !response.status().is_success() || !content_type(&response, "text/event-stream") {
        return Err(unexpected());
    }
    Ok(super::http_limits::sse(response, event_limit, remaining))
}

pub(super) fn delete(response: Response) -> Result<(), WireError> {
    if response.status() == StatusCode::METHOD_NOT_ALLOWED || response.status().is_success() {
        Ok(())
    } else {
        Err(unexpected())
    }
}
