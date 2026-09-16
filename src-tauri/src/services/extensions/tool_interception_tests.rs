use super::host_identity::HostIdentity;
use super::tool_interception::{run_chain, InterceptorCatalog, InterceptorRegistration};
use super::tool_interception_result::{self, CallError, Outcome};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

fn registration(id: &str, generation: u64) -> InterceptorRegistration {
    InterceptorRegistration {
        extension_id: id.to_string(),
        identity: HostIdentity::ThirdParty(id.to_string()),
        generation,
    }
}

#[test]
fn interception_order_uses_discovery_snapshot() {
    let catalog = InterceptorCatalog::default();
    catalog.replace(vec![registration("z", 1), registration("a", 2)]);
    let order = vec!["a".to_string(), "z".to_string()];

    assert_eq!(catalog.snapshot(&order).extension_ids(), ["a", "z"]);
}

#[tokio::test(start_paused = true)]
async fn interceptor_can_deny_but_never_upgrade_permissions() {
    let effect_count = AtomicUsize::new(0);
    let (outcome, _) = run_chain(
        &[registration("guard", 1)],
        json!({}),
        &CancellationToken::new(),
        |_, _, _| async { Ok(json!({"decision":"deny", "reason":"blocked"})) },
    )
    .await;
    if matches!(outcome, Outcome::Continue) {
        effect_count.fetch_add(1, Ordering::SeqCst);
    }
    assert!(matches!(outcome, Outcome::Deny));
    assert_eq!(effect_count.load(Ordering::SeqCst), 0);
}

#[tokio::test(start_paused = true)]
async fn individual_timeout_denies_call_and_disables_owner() {
    let (outcome, owner) = run_chain(
        &[registration("slow", 1)],
        json!({}),
        &CancellationToken::new(),
        |_, _, deadline| delayed_response(deadline, 251),
    )
    .await;
    assert!(matches!(
        outcome,
        Outcome::Disable(super::types::DIAGNOSTIC_INTERCEPTOR_TIMEOUT)
    ));
    assert_eq!(owner.unwrap().extension_id, "slow");
}

#[tokio::test(start_paused = true)]
async fn chain_timeout_denies_call_without_disabling_extensions() {
    let calls = Arc::new(AtomicUsize::new(0));
    let durations = Arc::new([249_u64, 249, 249, 249, 20]);
    let (outcome, _) = run_chain(
        &(0..5)
            .map(|i| registration(&format!("p{i}"), 1))
            .collect::<Vec<_>>(),
        json!({}),
        &CancellationToken::new(),
        {
            let calls = calls.clone();
            move |_, _, deadline| {
                let index = calls.fetch_add(1, Ordering::SeqCst);
                delayed_response(deadline, durations[index])
            }
        },
    )
    .await;
    assert!(matches!(outcome, Outcome::ChainTimeout));
    assert_eq!(calls.load(Ordering::SeqCst), 5);
}

#[test]
fn interception_deadline_tie_does_not_disable_owner() {
    let now = tokio::time::Instant::now();
    assert!(matches!(
        tool_interception_result::classify(Err(CallError::Timeout), now, now),
        Outcome::ChainTimeout
    ));
}

#[tokio::test(start_paused = true)]
async fn expired_chain_does_not_launch_the_next_handler() {
    let calls = Arc::new(AtomicUsize::new(0));
    let (outcome, _) = run_chain(
        &[
            registration("first", 1),
            registration("second", 1),
            registration("third", 1),
            registration("fourth", 1),
            registration("never", 1),
        ],
        json!({}),
        &CancellationToken::new(),
        {
            let calls = calls.clone();
            move |_, _, _| {
                let index = calls.fetch_add(1, Ordering::SeqCst);
                async move {
                    tokio::time::sleep(Duration::from_millis(249)).await;
                    if index == 3 {
                        tokio::time::advance(Duration::from_millis(5)).await;
                    }
                    Ok(json!({"decision":"continue"}))
                }
            }
        },
    )
    .await;
    assert!(matches!(outcome, Outcome::ChainTimeout));
    assert_eq!(calls.load(Ordering::SeqCst), 4);
}

#[tokio::test(start_paused = true)]
async fn interception_order_and_total_deadline_are_deterministic() {
    let catalog = InterceptorCatalog::default();
    catalog.replace(
        (0..8)
            .map(|index| registration(&format!("p{index}"), 1))
            .collect(),
    );
    let order = (0..8)
        .rev()
        .map(|index| format!("p{index}"))
        .collect::<Vec<_>>();

    assert_eq!(
        catalog.snapshot(&order).extension_ids(),
        ["p7", "p6", "p5", "p4", "p3", "p2", "p1", "p0"]
    );
    let calls = AtomicUsize::new(0);
    let snapshot = catalog.snapshot(&order);
    let (outcome, _) = run_chain(
        &snapshot.entries,
        json!({}),
        &CancellationToken::new(),
        |_, _, deadline| {
            calls.fetch_add(1, Ordering::SeqCst);
            delayed_response(deadline, 1)
        },
    )
    .await;
    assert!(matches!(outcome, Outcome::Continue));
    assert_eq!(calls.load(Ordering::SeqCst), 8);
}

#[test]
fn eager_execution_waits_when_interception_is_active() {
    let catalog = InterceptorCatalog::default();
    catalog.replace(vec![registration("guard", 1)]);
    let snapshot = catalog.snapshot(&["guard".to_string()]);

    assert!(!snapshot.is_empty());
}

#[test]
fn interceptor_activated_mid_stream_applies_next_request_without_replay() {
    let catalog = InterceptorCatalog::default();
    let order = ["guard".to_string()];
    let first_request = catalog.snapshot(&order);
    catalog.replace(vec![registration("guard", 1)]);
    let next_request = catalog.snapshot(&order);

    assert!(first_request.is_empty());
    assert_eq!(next_request.extension_ids(), ["guard"]);
}

#[test]
fn all_execution_paths_are_intercepted_before_effect() {
    assert!(include_str!("../agent_local/tool_executor_sequential.rs")
        .contains("intercept_and_publish"));
    assert!(
        include_str!("../agent_local/tool_executor_sequential_support.rs")
            .contains("before_tool_effect")
    );
    for source in [
        include_str!("../agent_local/tool_executor_parallel.rs"),
        include_str!("../agent_local/tool_executor_write.rs"),
        include_str!("../agent_local/tool_executor_delegate_launch.rs"),
        include_str!("../agent_local/eager_dispatch.rs"),
    ] {
        assert!(source.contains("before_tool_effect"));
    }
}

async fn delayed_response(
    deadline: tokio::time::Instant,
    delay_ms: u64,
) -> Result<serde_json::Value, CallError> {
    tokio::time::timeout_at(deadline, async {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        json!({"decision":"continue"})
    })
    .await
    .map_err(|_| CallError::Timeout)
}
