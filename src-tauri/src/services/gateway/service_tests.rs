use crate::models::GatewayConfig;
use crate::services::gateway::service::GatewayService;
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
