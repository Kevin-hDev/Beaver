use super::agent_work_supervision::AgentWorkServices;
use super::subagent_spawn_channel;
use crate::app_exit::AppExitCoordinator;
use std::future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[test]
fn full_channel_does_not_publish_spawned_event() {
    let (sender, _receiver) = mpsc::channel(1);
    sender.try_send(1).expect("fill channel");
    let published = AtomicBool::new(false);

    let error = subagent_spawn_channel::try_send_then(&sender, 2, || {
        published.store(true, Ordering::SeqCst)
    })
    .expect_err("full dispatcher");
    assert_eq!(error, "service-work-capacity-reached");
    assert!(!published.load(Ordering::SeqCst));
}

#[test]
fn closed_channel_does_not_publish_spawned_event() {
    let (sender, receiver) = mpsc::channel(1);
    drop(receiver);
    let published = AtomicBool::new(false);

    let error = subagent_spawn_channel::try_send_then(&sender, 1, || {
        published.store(true, Ordering::SeqCst)
    })
    .expect_err("closed dispatcher");
    assert_eq!(error, "service-shutting-down");
    assert!(!published.load(Ordering::SeqCst));
}

#[test]
fn accepted_request_publishes_once() {
    let (sender, _receiver) = mpsc::channel(1);
    let published = AtomicBool::new(false);

    subagent_spawn_channel::try_send_then(&sender, 1, || published.store(true, Ordering::SeqCst))
        .expect("accept request");
    assert!(published.load(Ordering::SeqCst));
}

#[tokio::test]
async fn tracked_child_shutdown_reaches_the_existing_request_token() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let services = AgentWorkServices::new(coordinator.work_supervisor());
    let admission = services.subagents().try_admit().expect("child admission");
    let request_cancel = CancellationToken::new();

    subagent_spawn_channel::spawn_tracked(
        admission,
        request_cancel.clone(),
        future::pending::<()>(),
    )
    .expect("tracked child starts");

    assert!(
        services
            .subagents()
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert!(request_cancel.is_cancelled());
}

#[tokio::test]
async fn dispatcher_shutdown_closes_its_bounded_queue() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let services = AgentWorkServices::new(coordinator.work_supervisor());
    let (sender, mut receiver) = mpsc::channel::<()>(1);

    services
        .subagent_dispatcher()
        .spawn(move |shutdown| async move {
            while subagent_spawn_channel::receive_next(&mut receiver, &shutdown)
                .await
                .is_some()
            {}
        })
        .expect("dispatcher starts");

    assert!(
        services
            .subagent_dispatcher()
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert!(sender.is_closed());
}
