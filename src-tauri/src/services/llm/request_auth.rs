use reqwest::header::HeaderValue;

use super::route_profile::ApiKeyHeader;

pub(in crate::services::llm) fn apply(
    request: reqwest::RequestBuilder,
    header: ApiKeyHeader,
    token: &str,
) -> reqwest::RequestBuilder {
    match header {
        ApiKeyHeader::Bearer => request.bearer_auth(token),
        ApiKeyHeader::XApiKey => apply_sensitive_x_api_key(request, token),
    }
}

fn apply_sensitive_x_api_key(
    request: reqwest::RequestBuilder,
    token: &str,
) -> reqwest::RequestBuilder {
    let Ok(mut value) = HeaderValue::from_bytes(token.as_bytes()) else {
        return request.header("x-api-key", token);
    };
    // Custom authentication headers need the same debug redaction as Authorization.
    value.set_sensitive(true);
    request.header("x-api-key", value)
}
