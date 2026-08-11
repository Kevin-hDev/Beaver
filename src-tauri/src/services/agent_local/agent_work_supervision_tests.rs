use super::agent_work_supervision::AgentWorkServices;
use crate::app_exit::AppExitCoordinator;
use crate::services::work_registry::ServiceWorkPhase;
use std::time::{Duration, Instant};

#[tokio::test]
async fn one_owner_stops_every_agent_work_class_on_one_deadline() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let services = AgentWorkServices::new(coordinator.work_supervisor());
    services
        .streams()
        .spawn(|cancel| async move { cancel.cancelled().await })
        .expect("stream task");
    services
        .subagents()
        .spawn(|cancel| async move { cancel.cancelled().await })
        .expect("subagent task");
    services
        .shells()
        .spawn(|cancel| async move { cancel.cancelled().await })
        .expect("shell task");
    services
        .subagent_dispatcher()
        .spawn(|cancel| async move { cancel.cancelled().await })
        .expect("dispatcher task");

    assert!(
        services
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    for service in [
        services.streams().phase(),
        services.subagents().phase(),
        services.shells().phase(),
        services.subagent_dispatcher().phase(),
    ] {
        assert_eq!(service, ServiceWorkPhase::Closed);
    }
}
