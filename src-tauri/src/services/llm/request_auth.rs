use super::route_profile::ApiKeyHeader;

pub(in crate::services::llm) fn apply(
    request: reqwest::RequestBuilder,
    header: ApiKeyHeader,
    token: &str,
) -> reqwest::RequestBuilder {
    match header {
        ApiKeyHeader::Bearer => request.bearer_auth(token),
        ApiKeyHeader::XApiKey => {
            crate::services::secure_http::sensitive_header(request, "x-api-key", token)
        }
    }
}
