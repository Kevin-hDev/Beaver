use crate::models::GatewayConfig;
use crate::services::gateway::service::GatewayService;

#[tokio::test]
async fn new_service_is_not_enabled() {
    let svc = GatewayService::new();
    assert!(!svc.is_enabled().await);
}

#[tokio::test]
async fn update_config_persists() {
    let svc = GatewayService::new();
    let cfg = GatewayConfig {
        enabled: true,
        max_sessions: 42,
        ..Default::default()
    };
    svc.update_config(cfg).await;
    assert_eq!(svc.config().await.max_sessions, 42);
}

#[tokio::test]
async fn stop_cancels_and_restart_works() {
    let svc = GatewayService::new();
    assert!(!svc.state.read().await.cancel.is_cancelled());
    svc.stop().await;
    assert!(svc.state.read().await.cancel.is_cancelled());
}
