use reqwest::{Response, StatusCode};
use serde_json::Value;
use std::time::Duration;
use zeroize::Zeroizing;

use crate::services::secure_http::{AuthenticatedClient, SecureHttpError};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const RETRY_DELAYS_SECS: [u64; 2] = [2, 4];

pub async fn post_json_with_retry(
    client: &AuthenticatedClient,
    endpoint: &str,
    api_key: &Zeroizing<String>,
    payload: &Value,
) -> Result<Response, String> {
    for attempt in 0..=RETRY_DELAYS_SECS.len() {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_secs(RETRY_DELAYS_SECS[attempt - 1])).await;
        }

        match send_once(client, endpoint, api_key, payload).await {
            Ok(resp) if resp.status().is_success() => return Ok(resp),
            Ok(resp) if should_retry_status(resp.status()) && attempt < RETRY_DELAYS_SECS.len() => {
            }
            Ok(_) => return Err("Erreur du service de prédiction".to_string()),
            Err(error) if should_retry_error(&error) && attempt < RETRY_DELAYS_SECS.len() => {}
            Err(_) => return Err("Erreur du service de prédiction".to_string()),
        }
    }

    Err("Erreur du service de prédiction".to_string())
}

async fn send_once(
    client: &AuthenticatedClient,
    endpoint: &str,
    api_key: &Zeroizing<String>,
    payload: &Value,
) -> Result<Response, SecureHttpError> {
    let request = client
        .post(endpoint)
        .bearer_auth(api_key.as_str())
        .header("Content-Type", "application/json")
        .json(payload)
        .timeout(REQUEST_TIMEOUT);
    client.send(request).await
}

fn should_retry_status(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::TOO_MANY_REQUESTS
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT
    )
}

fn should_retry_error(error: &SecureHttpError) -> bool {
    matches!(error, SecureHttpError::Request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn redirect_never_receives_forecast_secret_or_payload() {
        let destination = MockServer::start().await;
        let origin = MockServer::start().await;
        Mock::given(any())
            .respond_with(
                ResponseTemplate::new(307)
                    .insert_header("Location", format!("{}/sink", destination.uri())),
            )
            .mount(&origin)
            .await;
        let client = AuthenticatedClient::new_loopback(Duration::from_secs(2)).unwrap();
        let key = Zeroizing::new("fixture-forecast-key".to_string());

        let result = post_json_with_retry(
            &client,
            &format!("{}/forecast", origin.uri()),
            &key,
            &serde_json::json!({"secret_payload": true}),
        )
        .await;

        assert!(result.is_err());
        assert!(destination.received_requests().await.unwrap().is_empty());
    }
}
