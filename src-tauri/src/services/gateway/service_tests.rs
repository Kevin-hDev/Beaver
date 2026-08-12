use crate::models::GatewayConfig;
use crate::services::gateway::service::GatewayService;
use crate::services::gateway::service_state::ChannelEntry;
use crate::services::gateway::types::{ChannelKey, ChannelStatus};
use crate::services::gateway::work_supervision::GatewayWorkServices;
use crate::services::work_registry::ServiceWorkPhase;
use std::time::{Duration, Instant};

fn service() -> GatewayService {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().expect("exit coordinator");
    GatewayService::new(coordinator.work_supervisor())
}

#[tokio::test]
async fn new_service_is_not_enabled() {
    let svc = service();
    assert!(!svc.is_enabled().await);
    assert!(!svc.health().await.running);
}

#[tokio::test]
async fn update_config_persists() {
    let svc = service();
    let cfg = GatewayConfig {
        enabled: true,
        max_sessions: 42,
        ..Default::default()
    };
    svc.update_config(cfg).await;
    assert_eq!(svc.config().await.max_sessions, 42);
    assert!(!svc.is_enabled().await, "configuration alone is not a run");
    assert!(!svc.health().await.running);
}

#[tokio::test]
async fn stop_is_idempotent_before_any_run() {
    let svc = service();
    assert!(svc.state.read().await.cancel.is_cancelled());
    assert!(
        svc.stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert!(svc.state.read().await.cancel.is_cancelled());
    assert!(
        svc.stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
}

#[tokio::test]
async fn stop_cancels_the_active_run_before_waiting_for_the_run_lock() {
    let svc = service();
    let cancel = tokio_util::sync::CancellationToken::new();
    svc.set_active_cancel_for_test(cancel.clone());
    let _run_guard = svc.run.lock().await;

    assert!(
        !svc.stop_and_wait(Instant::now() + Duration::from_millis(20))
            .await
    );
    assert!(cancel.is_cancelled());
}

#[tokio::test]
async fn successful_stop_publishes_off_for_every_channel() {
    let svc = service();
    let key = ChannelKey::new("discord", "main");
    svc.state.write().await.channels.insert(
        key.clone(),
        ChannelEntry {
            status: ChannelStatus::Running,
            cancel: tokio_util::sync::CancellationToken::new(),
            error: None,
        },
    );

    assert!(
        svc.stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert_eq!(
        svc.state.read().await.channels[&key].status,
        ChannelStatus::Off
    );
}

#[tokio::test]
async fn blocked_audit_never_holds_the_gateway_state_lock() {
    let svc = service();
    let key = ChannelKey::new("discord", "audit");
    svc.state.write().await.channels.insert(
        key,
        ChannelEntry {
            status: ChannelStatus::Running,
            cancel: tokio_util::sync::CancellationToken::new(),
            error: None,
        },
    );
    let (audit_started_tx, audit_started_rx) = tokio::sync::oneshot::channel();
    let (resume_audit_tx, resume_audit_rx) = tokio::sync::oneshot::channel();
    let stop = svc.stop_and_wait_with_audit_for_test(
        Instant::now() + Duration::from_secs(1),
        move |_| async move {
            let _ = audit_started_tx.send(());
            let _ = resume_audit_rx.await;
            true
        },
    );
    tokio::pin!(stop);
    tokio::select! {
        _ = audit_started_rx => {}
        result = &mut stop => panic!("stop completed before blocked audit: {result}"),
    }

    tokio::time::timeout(Duration::from_millis(20), svc.health())
        .await
        .expect("health must not wait for audit");
    resume_audit_tx.send(()).unwrap();
    assert!(stop.await);
}

#[tokio::test]
async fn admission_is_closed_before_the_stop_audit_runs() {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().expect("exit coordinator");
    let svc = GatewayService::new(coordinator.work_supervisor());
    let work = GatewayWorkServices::new(coordinator.work_supervisor());
    *svc.run.lock().await = Some(work.clone());
    let (audit_started_tx, audit_started_rx) = tokio::sync::oneshot::channel();
    let (resume_audit_tx, resume_audit_rx) = tokio::sync::oneshot::channel();
    let stop = svc.stop_and_wait_with_audit_for_test(
        Instant::now() + Duration::from_secs(1),
        move |_| async move {
            let _ = audit_started_tx.send(());
            let _ = resume_audit_rx.await;
            true
        },
    );
    tokio::pin!(stop);
    tokio::select! {
        _ = audit_started_rx => {}
        result = &mut stop => panic!("stop completed before blocked audit: {result}"),
    }

    assert_ne!(work.consumer_phase(), ServiceWorkPhase::Open);
    assert_ne!(work.channel_phase(), ServiceWorkPhase::Open);
    assert_ne!(work.message_phase(), ServiceWorkPhase::Open);
    resume_audit_tx.send(()).unwrap();
    assert!(stop.await);
}
