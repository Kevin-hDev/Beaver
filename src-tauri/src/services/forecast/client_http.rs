use std::time::Duration;

use serde::de::DeserializeOwned;

use crate::services::secure_http::{read_json_bounded, AuthenticatedClient};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
const NETWORK_ERROR: &str = "Erreur du service de prédiction";
const RESPONSE_ERROR: &str = "Réponse du service de prédiction invalide";

pub fn internet_client() -> Result<AuthenticatedClient, String> {
    AuthenticatedClient::new(REQUEST_TIMEOUT).map_err(|_| NETWORK_ERROR.to_string())
}

pub fn loopback_client() -> Result<AuthenticatedClient, String> {
    AuthenticatedClient::new_loopback(REQUEST_TIMEOUT).map_err(|_| NETWORK_ERROR.to_string())
}

pub async fn read_json<T: DeserializeOwned>(response: reqwest::Response) -> Result<T, String> {
    read_json_bounded(
        response,
        crate::services::forecast::limits::MAX_RESPONSE_BYTES,
    )
    .await
    .map_err(|_| RESPONSE_ERROR.to_string())
}
