use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

pub const PROXY_BASE_URL: &str = "https://cli-chat-proxy.grok.com/v1";

pub fn insert_identity(headers: &mut HeaderMap) -> Result<(), String> {
    insert(headers, "x-xai-token-auth", "xai-grok-cli")?;
    insert(headers, "x-authenticateresponse", "authenticate-response")?;
    insert(headers, "x-grok-client-identifier", "Beaver")?;
    insert(headers, "x-grok-client-version", env!("CARGO_PKG_VERSION"))?;
    insert(headers, "x-grok-client-mode", "interactive")
}

pub fn insert_user(headers: &mut HeaderMap, user_id: &str) -> Result<(), String> {
    insert(headers, "x-userid", user_id)
}

pub fn model_header(model: &str) -> Result<HeaderMap, String> {
    if !crate::services::llm::runtime_models::valid_model_id(model) {
        return Err(unavailable());
    }
    let mut headers = HeaderMap::new();
    insert(&mut headers, "x-grok-model-override", model)?;
    Ok(headers)
}

fn insert(headers: &mut HeaderMap, name: &'static str, value: &str) -> Result<(), String> {
    let name = HeaderName::from_static(name);
    let value = HeaderValue::from_str(value).map_err(|_| unavailable())?;
    headers.insert(name, value);
    Ok(())
}

fn unavailable() -> String {
    "provider_configuration_invalid".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::llm::request_purpose::RequestPurpose;
    use crate::services::llm_oauth::{headers, LlmOAuthProvider};
    use reqwest::header::USER_AGENT;

    #[test]
    fn proxy_identity_is_internal_and_truthful() {
        let values = headers::request_headers_with_identity(
            LlmOAuthProvider::Xai,
            RequestPurpose::ManualChat,
            Some("fixture-user"),
        )
        .unwrap();
        assert_eq!(values["x-xai-token-auth"], "xai-grok-cli");
        assert_eq!(values["x-grok-client-identifier"], "Beaver");
        assert_eq!(values["x-grok-client-version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(values["x-userid"], "fixture-user");
        assert_eq!(values[USER_AGENT], headers::user_agent());
        assert_eq!(
            model_header("grok-4.6").unwrap()["x-grok-model-override"],
            "grok-4.6"
        );
        assert_eq!(
            model_header("../invalid").unwrap_err(),
            "provider_configuration_invalid"
        );
    }
}
