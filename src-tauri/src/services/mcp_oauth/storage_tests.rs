use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

use super::storage::{get_valid_token_with, RefreshDependencies};
use super::types::OAuthTokens;

struct MemoryTokens {
    generation: u64,
    json: Zeroizing<String>,
    writes: usize,
}

#[test]
fn stale_refresh_generation_never_writes() {
    let wrote = AtomicBool::new(false);
    let result = super::storage::save_if_generation(u64::MAX, || {
        wrote.store(true, Ordering::SeqCst);
        Ok(())
    });
    assert!(result.is_err());
    assert!(!wrote.load(Ordering::SeqCst));
}

#[tokio::test]
async fn oauth_refresh_keeps_generation_or_rejects_stale_write() {
    for disconnect_during_refresh in [false, true] {
        let server = wiremock::MockServer::start().await;
        let endpoint = format!("{}/token", server.uri());
        let initial = OAuthTokens {
            access_token: Zeroizing::new("old-access".to_string()),
            refresh_token: Some(Zeroizing::new("old-refresh".to_string())),
            expires_at: Some(chrono::Utc::now().timestamp().saturating_sub(60)),
            token_type: "Bearer".to_string(),
            token_endpoint: endpoint.clone(),
            client_id: "fixture-client".to_string(),
            client_secret: None,
        };
        let memory = Arc::new(Mutex::new(MemoryTokens {
            generation: 7,
            json: initial.to_json().unwrap(),
            writes: 0,
        }));
        let response_memory = memory.clone();
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .and(wiremock::matchers::path("/token"))
            .respond_with(move |_: &wiremock::Request| {
                if disconnect_during_refresh {
                    response_memory.lock().unwrap().generation = 8;
                }
                wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "access_token":"new-access", "refresh_token":"new-refresh",
                    "expires_in":3600, "token_type":"Bearer"
                }))
            })
            .expect(1)
            .mount(&server)
            .await;

        let read_memory = memory.clone();
        let read = move |_: &str| OAuthTokens::from_json(read_memory.lock().unwrap().json.as_str());
        let current_memory = memory.clone();
        let read_current = move |_: &str, generation: u64| {
            let state = current_memory.lock().unwrap();
            if state.generation != generation {
                return Err("identité MCP modifiée".to_string());
            }
            OAuthTokens::from_json(state.json.as_str())
        };
        let save_memory = memory.clone();
        let save = move |_: &str, tokens: &OAuthTokens, generation: u64| {
            let mut state = save_memory.lock().unwrap();
            if state.generation != generation {
                return Err("identité MCP modifiée".to_string());
            }
            state.json = tokens.to_json()?;
            state.writes += 1;
            Ok(())
        };
        let generation_memory = memory.clone();
        let generation = move || Ok(generation_memory.lock().unwrap().generation);
        let trusted_endpoint = endpoint.clone();
        let validate = move |id: &str, url: &str| {
            if id == "test-refresh" && url == trusted_endpoint {
                Ok(())
            } else {
                Err("endpoint OAuth refusé".to_string())
            }
        };
        let client = || {
            crate::services::secure_http::AuthenticatedClient::new_loopback(
                std::time::Duration::from_secs(2),
            )
            .map_err(|_| "client test indisponible".to_string())
        };
        let dependencies = RefreshDependencies {
            read: &read,
            read_current: &read_current,
            save: &save,
            generation: &generation,
            validate: &validate,
            client: &client,
        };
        let result = get_valid_token_with("test-refresh", &dependencies).await;
        {
            let state = memory.lock().unwrap();
            if disconnect_during_refresh {
                assert!(result.is_err());
                assert_eq!(state.writes, 0);
            } else {
                let token = result.expect("same-account refresh");
                assert!(bool::from(token.as_bytes().ct_eq(b"new-access")));
                assert_eq!(state.generation, 7);
                assert_eq!(state.writes, 1);
            }
        }
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }
}
