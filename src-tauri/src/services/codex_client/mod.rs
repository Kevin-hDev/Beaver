pub mod convert;
mod http_error;
mod limits;
pub mod model_catalog;
mod model_catalog_fallback;
mod model_catalog_fast;
mod model_catalog_wire;
pub mod request;
mod request_build;
mod request_http;
mod routing_hint;
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

#[cfg(test)]
#[path = "test_transport.rs"]
pub(crate) mod test_transport;

#[cfg(test)]
#[path = "reasoning_continuity_tests.rs"]
mod reasoning_continuity_tests;
#[cfg(test)]
#[path = "transport_orchestration_tests.rs"]
mod transport_orchestration_tests;

pub const PROVIDER_ID: &str = "codex-oauth";

/// Le protocole du catalogue Codex ne publie pas encore cette capacité : tous
/// les modèles Codex servis acceptent les outils. Lire le champ du protocole
/// ici si l'API en expose un à l'avenir.
pub fn supports_tools(_model_id: &str) -> bool {
    true
}
