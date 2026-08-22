pub mod convert;
mod http_error;
mod limits;
pub mod model_catalog;
mod model_catalog_fallback;
mod model_catalog_fast;
mod model_catalog_wire;
mod replay;
pub mod request;
mod request_http;
pub mod stream;
mod stream_accumulator;
mod stream_measurement;
mod stream_protocol;
pub mod stream_silent;
mod stream_tool;
pub mod types;
mod websocket;
mod websocket_connect;
mod websocket_url;

pub const PROVIDER_ID: &str = "codex-oauth";

/// Le protocole du catalogue Codex ne publie pas encore cette capacité : tous
/// les modèles Codex servis acceptent les outils. Lire le champ du protocole
/// ici si l'API en expose un à l'avenir.
pub fn supports_tools(_model_id: &str) -> bool {
    true
}
