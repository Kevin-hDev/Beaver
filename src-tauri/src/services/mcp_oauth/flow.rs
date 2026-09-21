use std::collections::HashMap;
use std::sync::Mutex;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64URL, Engine};
use tauri::Emitter;
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;

use crate::services::work_registry::ServiceWorkCancellation;

use super::{callback_server, discovery, flow_auth, pkce, static_credentials, storage};

const MAX_PENDING: usize = 5;
const CANCELLED_MSG: &str = "annulé";
const ALREADY_RUNNING: &str = "already_running";

static PENDING: std::sync::LazyLock<Mutex<HashMap<String, CancellationToken>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn run(
    app: tauri::AppHandle,
    connector_id: String,
    endpoint: String,
    work_cancel: ServiceWorkCancellation,
) {
    let result = tokio::select! {
        result = run_inner(&app, &connector_id, &endpoint) => result,
        _ = work_cancel.cancelled() => Err(CANCELLED_MSG.to_string()),
    };
    let skip_emit = matches!(&result, Err(e) if e == CANCELLED_MSG || e == ALREADY_RUNNING);
    if !skip_emit {
        let (success, error) = match &result {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e.as_str())),
        };
        let _ = app.emit(
            "mcp-oauth-result",
            serde_json::json!({
                "connector_id": connector_id,
                "success": success,
                "error": error,
            }),
        );
    }
    if !matches!(&result, Err(e) if e == ALREADY_RUNNING) {
        cleanup_pending(&connector_id);
    }
}

async fn run_inner(
    app: &tauri::AppHandle,
    connector_id: &str,
    endpoint: &str,
) -> Result<(), String> {
    let cancel = register_pending(connector_id)?;
    // ponytail: the registry generation is global; an unrelated edit may require retrying OAuth.
    // Use per-connector generations only if that becomes a real usability problem.
    let expected_generation = crate::services::mcp_bridge::registry::current_generation()?;

    let meta = discovery::discover_auth_server(connector_id, endpoint).await?;

    let server = callback_server::CallbackServer::bind().await?;
    let port = server.port();
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");

    let static_creds = static_credentials::for_endpoint(endpoint);

    let (client_id, client_secret, scopes) = if let Some(ref creds) = static_creds {
        (
            creds.client_id.to_string(),
            Some(Zeroizing::new(creds.client_secret.to_string())),
            Some(creds.scopes),
        )
    } else if let Some(ref reg_url) = meta.registration_endpoint {
        let id = flow_auth::register_client(connector_id, endpoint, reg_url, &redirect_uri).await?;
        (id, None, None)
    } else {
        return Err("pas de credentials disponibles pour ce service".to_string());
    };

    let (verifier, challenge) = pkce::generate();
    let state = generate_state();

    let auth_url = flow_auth::build_auth_url(
        &meta.authorization_endpoint,
        &client_id,
        &redirect_uri,
        &challenge,
        &state,
        endpoint,
        scopes,
    )?;

    flow_auth::open_browser(app, connector_id, &auth_url)?;

    let callback = server.wait(&state, &cancel).await?;

    flow_auth::verify_state_constant_time(&state, &callback.state)?;
    super::issuer::verify_callback_issuer(
        &meta.issuer,
        callback.iss.as_deref(),
        meta.authorization_response_iss_parameter_supported,
    )?;

    let secret_ref = client_secret.as_ref().map(|s| s.as_str());
    let tokens = flow_auth::exchange_code(
        connector_id,
        &meta.issuer,
        &meta.token_endpoint,
        &callback.code,
        &client_id,
        secret_ref,
        &verifier,
        &redirect_uri,
        endpoint,
    )
    .await?;

    drop(verifier);
    let connector = crate::services::mcp_bridge::config::StoredConnector {
        id: connector_id.to_string(),
        status: "connected".to_string(),
        enabled_in_chat: true,
        endpoint: Some(endpoint.to_string()),
        install_command: None,
        env_keys: None,
    };
    crate::services::mcp_bridge::registry::test_connector_with_oauth_token(
        connector.clone(),
        tokens.access_token.clone(),
    )
    .await?;
    if cancel.is_cancelled() {
        return Err(CANCELLED_MSG.to_string());
    }
    crate::services::mcp_bridge::registry_commit::commit_after_probe(
        connector_id,
        Some(expected_generation),
        Ok(()),
        || crate::services::mcp_bridge::config::find(connector_id),
        || {
            if cancel.is_cancelled() {
                return Err(CANCELLED_MSG.to_string());
            }
            crate::services::mcp_bridge::config::upsert(connector)
        },
        |mutation| {
            if cancel.is_cancelled() {
                return Err(CANCELLED_MSG.to_string());
            }
            storage::store_connection_tokens(mutation, connector_id, &tokens)
        },
        |previous| crate::services::mcp_bridge::config::restore(connector_id, previous),
    )?;
    let _ = app.emit("fs:connectors-changed", ());
    Ok(())
}

fn register_pending(connector_id: &str) -> Result<CancellationToken, String> {
    let mut map = PENDING.lock().map_err(|_| "erreur interne".to_string())?;
    if map.contains_key(connector_id) {
        return Err(ALREADY_RUNNING.to_string());
    }
    if map.len() >= MAX_PENDING {
        return Err("trop de flows OAuth en cours".to_string());
    }
    let token = CancellationToken::new();
    map.insert(connector_id.to_string(), token.clone());
    Ok(token)
}

fn cleanup_pending(connector_id: &str) {
    if let Ok(mut map) = PENDING.lock() {
        map.remove(connector_id);
    }
}

pub fn cancel(connector_id: &str) {
    if let Ok(map) = PENDING.lock() {
        if let Some(token) = map.get(connector_id) {
            token.cancel();
        }
    }
}

fn generate_state() -> Zeroizing<String> {
    let mut bytes = [0u8; 32];
    crate::services::secure_random::fill(&mut bytes);
    let s = B64URL.encode(bytes);
    bytes.fill(0);
    Zeroizing::new(s)
}
