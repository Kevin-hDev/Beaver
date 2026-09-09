use super::{authorize_payload, authorize_serialized_len, output_limit, run_scoped, FixtureLimits};
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

#[test]
fn malformed_or_out_of_range_fixture_limits_are_rejected() {
    assert!(FixtureLimits::from_values(Some("0"), None, None).is_err());
    assert!(FixtureLimits::from_values(Some("8193"), None, None).is_err());
    assert!(FixtureLimits::from_values(None, Some("5"), None).is_err());
    assert!(FixtureLimits::from_values(None, Some("0"), Some("65537")).is_err());
    assert!(FixtureLimits::from_values(Some("bad"), None, None).is_err());
}

#[test]
fn fixture_limits_preserve_defaults_and_smaller_existing_caps() {
    let limits = FixtureLimits::from_values(None, None, None).unwrap();
    assert_eq!(limits.output_tokens, 512);
    assert_eq!(limits.attempts, 4);
    assert_eq!(limits.input_bytes, 8_192);
}

#[tokio::test]
async fn budget_counts_attempts_and_cleans_up_after_error() {
    let limits = FixtureLimits::from_values(Some("8"), Some("2"), Some("64")).unwrap();
    let token = CancellationToken::new();
    let result = run_scoped(limits, token, async {
        assert_eq!(output_limit(None), Some(8));
        assert_eq!(output_limit(Some(3)), Some(3));
        authorize_serialized_len(16)?;
        authorize_serialized_len(16)?;
        assert!(authorize_serialized_len(16).is_err());
        Err::<(), String>("fixture error".to_string())
    })
    .await;
    assert_eq!(result, Err("fixture error".to_string()));
    assert_eq!(output_limit(None), None);
    assert!(authorize_serialized_len(16).is_ok());
}

#[tokio::test]
async fn oversized_payload_is_rejected_before_consuming_a_generation_attempt() {
    let limits = FixtureLimits::from_values(Some("8"), Some("1"), Some("8")).unwrap();
    run_scoped(limits, CancellationToken::new(), async {
        assert!(authorize_payload(&serde_json::json!({"opaque": "too large"})).is_err());
        assert!(authorize_serialized_len(8).is_ok());
        assert!(authorize_serialized_len(8).is_err());
        Ok::<(), String>(())
    })
    .await
    .unwrap();
}

#[tokio::test(start_paused = true)]
async fn timeout_drops_pending_fixture_future_and_cancels_token() {
    let limits = FixtureLimits::from_values(Some("8"), Some("1"), Some("64")).unwrap();
    let token = CancellationToken::new();
    let observed = Arc::new(Mutex::new(false));
    let guard = DropGuard(observed.clone());
    let result_future = run_scoped(limits, token.clone(), pending::<()>(guard));
    tokio::pin!(result_future);
    tokio::task::yield_now().await;
    tokio::time::advance(super::TURN_TIMEOUT).await;
    let result = result_future.await;
    assert_eq!(result, Err("fixture timeout".to_string()));
    assert!(token.is_cancelled());
    assert!(*observed.lock().unwrap());
}

#[tokio::test]
async fn normal_task_has_no_fixture_budget() {
    assert_eq!(output_limit(None), None);
    assert_eq!(output_limit(Some(128)), Some(128));
    assert!(authorize_serialized_len(65_537).is_ok());
}

#[tokio::test]
async fn concurrent_normal_task_keeps_its_limits() {
    let limits = FixtureLimits::from_values(Some("8"), Some("1"), None).unwrap();
    run_scoped(limits, CancellationToken::new(), async {
        assert_eq!(output_limit(Some(128)), Some(8));
        tokio::spawn(async {
            assert_eq!(output_limit(Some(128)), Some(128));
            assert!(authorize_serialized_len(100_000).is_ok());
        })
        .await
        .unwrap();
        Ok(())
    })
    .await
    .unwrap();
}

#[test]
fn fixture_admission_rejects_unbudgeted_transports() {
    use crate::services::llm::route_profile::supports_bounded_fixture;
    for provider in [
        "google",
        "zai",
        "openai",
        "openrouter",
        "codex-oauth",
        "ollama",
        "qwen",
        "anthropic",
        "xai-oauth",
    ] {
        assert!(supports_bounded_fixture(provider), "{provider}");
    }
    for provider in ["moonshot-oauth", "unknown"] {
        assert!(!supports_bounded_fixture(provider), "{provider}");
    }
}

#[tokio::test(start_paused = true)]
async fn external_cancel_drops_pending_fixture_before_deadline() {
    let limits = FixtureLimits::from_values(None, None, None).unwrap();
    let token = CancellationToken::new();
    let observed = Arc::new(Mutex::new(false));
    let guard = DropGuard(observed.clone());
    let cancel = token.clone();
    let run = run_scoped(limits, token, pending::<()>(guard));
    let (result, ()) = tokio::join!(
        tokio::time::timeout(std::time::Duration::from_secs(1), run),
        async move {
            tokio::task::yield_now().await;
            cancel.cancel();
        }
    );
    assert_eq!(result, Ok(Err("Annulé".to_string())));
    assert!(*observed.lock().unwrap());
}

async fn pending<T>(guard: DropGuard) -> Result<T, String> {
    std::future::pending::<()>().await;
    drop(guard);
    unreachable!()
}

struct DropGuard(Arc<Mutex<bool>>);

impl Drop for DropGuard {
    fn drop(&mut self) {
        *self.0.lock().unwrap() = true;
    }
}
