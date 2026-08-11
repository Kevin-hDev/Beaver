use super::agent_chat_work;
use crate::app_exit::AppExitCoordinator;
use crate::services::agent_local::agent_work_supervision::AgentWorkServices;
use std::future;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn stream_shutdown_reaches_the_existing_request_token() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let work = AgentWorkServices::new(coordinator.work_supervisor());
    let admission = agent_chat_work::admit(&work).expect("stream admission");
    let request_cancel = CancellationToken::new();
    agent_chat_work::spawn(admission, request_cancel.clone(), future::pending::<()>())
        .expect("stream task starts");

    assert!(
        work.streams()
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert!(request_cancel.is_cancelled());
}

#[tokio::test]
async fn closed_stream_service_returns_a_stable_public_code() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let work = AgentWorkServices::new(coordinator.work_supervisor());
    assert!(
        work.streams()
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );

    assert_eq!(
        agent_chat_work::admit(&work).expect_err("closed stream service"),
        "service-shutting-down"
    );
}
