use serde_json::{json, Value};

use crate::services::brand::MCP_CLIENT_NAME;

pub(super) fn client_info() -> Value {
    json!({
        "name": MCP_CLIENT_NAME,
        "version": env!("CARGO_PKG_VERSION"),
    })
}

pub(crate) fn registration_name(connector_id: &str) -> String {
    format!("{MCP_CLIENT_NAME} ({connector_id})")
}
