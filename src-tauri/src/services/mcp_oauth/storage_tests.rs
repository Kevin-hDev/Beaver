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

#[tokio::test]
async fn legacy_token_requires_reauthentication_before_use() {
    let legacy = r#"{"access_token":"legacy-access","refresh_token":null,"expires_at":null,"token_type":"Bearer","token_endpoint":"https://github.com/login/oauth/access_token","client_id":"legacy"}"#;
    let read = |_: &str| OAuthTokens::from_json(legacy);
    let read_current = |_: &str, _: u64| OAuthTokens::from_json(legacy);
    let save = |_: &str, _: &OAuthTokens, _: u64| Ok(());
    let generation = || Ok(1);
    let validate = |_: &str, _: &str| Ok(());
    let client = |_: &str, _: &str| {
        Box::pin(async { Err("unused".to_string()) }) as super::storage::DestinationFuture
    };
    let dependencies = RefreshDependencies {
        read: &read,
        read_current: &read_current,
        save: &save,
        generation: &generation,
        validate: &validate,
        validate_issuer: &validate,
        client: &client,
    };
    assert_eq!(
        get_valid_token_with("github", &dependencies)
            .await
            .err()
            .as_deref(),
        Some("mcp_reauthentication_required")
    );
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

async fn refresh_case(disconnect_during_refresh: bool, status: u16, concurrent: bool) {
    let server = wiremock::MockServer::start().await;
    let endpoint = format!("{}/token", server.uri());
    let initial = OAuthTokens {
        issuer: Some(endpoint.clone()),
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
            wiremock::ResponseTemplate::new(status).set_body_json(serde_json::json!({
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
    let expected_destination = endpoint.clone();
    let client = move |id: &str, url: &str| {
        let id = id.to_string();
        let url = url.to_string();
        let expected = expected_destination.clone();
        Box::pin(async move {
            super::network_guard::destination_loopback_for_test(&id, &expected, &url).await
        }) as super::storage::DestinationFuture
    };
    let dependencies = RefreshDependencies {
        read: &read,
        read_current: &read_current,
        save: &save,
        generation: &generation,
        validate: &validate,
        validate_issuer: &validate,
        client: &client,
    };
    let result = if concurrent {
        let (first, second) = tokio::join!(
            get_valid_token_with("test-refresh", &dependencies),
            get_valid_token_with("test-refresh", &dependencies),
        );
        assert_eq!(second.unwrap().as_str(), "new-access");
        first
    } else {
        get_valid_token_with("test-refresh", &dependencies).await
    };
    {
        let state = memory.lock().unwrap();
        if disconnect_during_refresh {
            assert!(result.is_err());
            assert_eq!(state.writes, 0);
        } else if status == 401 || status == 403 {
            assert_eq!(
                result.err().as_deref(),
                Some("mcp_reauthentication_required")
            );
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

#[tokio::test]
async fn same_account_refresh_keeps_generation() {
    refresh_case(false, 200, false).await;
}

#[tokio::test]
async fn refresh_after_disconnect_cannot_restore_tokens() {
    refresh_case(true, 200, false).await;
}

#[tokio::test]
async fn rejected_refresh_requires_reauthentication() {
    refresh_case(false, 401, false).await;
    refresh_case(false, 403, false).await;
}

#[tokio::test]
async fn two_simultaneous_refreshes_use_one_network_exchange() {
    refresh_case(false, 200, true).await;
}
