pub mod callback_server;
pub mod discovery;
pub mod flow;
pub mod flow_auth;
pub mod pkce;
pub mod static_credentials;
pub mod storage;
pub mod trusted_oauth;
pub mod types;

pub(crate) async fn bounded_json<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, String> {
    crate::services::secure_http::read_json_bounded(
        resp,
        crate::services::secure_http::OAUTH_BODY_LIMIT,
    )
    .await
    .map_err(|_| "réponse OAuth invalide".to_string())
}
