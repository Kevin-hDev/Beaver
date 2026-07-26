use crate::services::{
    brand,
    secure_http::{read_bounded, read_json_bounded, AuthenticatedClient},
};
use serde_json::Value;
use std::time::Duration;

use super::limits::{MAX_MODEL_DETAILS_MARKDOWN_BYTES, MAX_MODEL_DETAILS_METADATA_BYTES};

pub fn client() -> Result<AuthenticatedClient, String> {
    AuthenticatedClient::new(Duration::from_secs(15))
        .map_err(|_| "Impossible de préparer la requête modèle".to_string())
}

pub async fn metadata(client: &AuthenticatedClient, url: &str) -> Result<Value, String> {
    let response = client
        .send_success(client.get(url).header("User-Agent", brand::user_agent()))
        .await
        .map_err(|_| "Impossible de charger les métadonnées du modèle".to_string())?;
    read_json_bounded(response, MAX_MODEL_DETAILS_METADATA_BYTES)
        .await
        .map_err(|_| "Impossible de lire les métadonnées du modèle".to_string())
}

pub async fn optional_markdown(client: &AuthenticatedClient, url: &str) -> String {
    let Ok(response) = client
        .send_success(client.get(url).header("User-Agent", brand::user_agent()))
        .await
    else {
        return String::new();
    };
    let Ok(body) = read_bounded(response, MAX_MODEL_DETAILS_MARKDOWN_BYTES).await else {
        return String::new();
    };
    String::from_utf8(body.to_vec()).unwrap_or_default()
}
