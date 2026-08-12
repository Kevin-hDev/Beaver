use super::work_supervision::{
    GatewayMessageAdmissionError, GatewayWorkServices, GATEWAY_MESSAGE_QUEUE_CAPACITY,
    MAX_ACTIVE_GATEWAY_CHANNELS, MAX_ACTIVE_GATEWAY_MESSAGES,
};
use crate::app_exit::AppExitCoordinator;
use crate::services::work_registry::ServiceWorkPhase;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn gateway_work() -> GatewayWorkServices {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    GatewayWorkServices::new(coordinator.work_supervisor())
}

#[test]
fn message_queue_is_bounded_to_the_contract() {
    let (sender, _receiver) = tokio::sync::mpsc::channel::<u8>(GATEWAY_MESSAGE_QUEUE_CAPACITY);
    for value in 0..GATEWAY_MESSAGE_QUEUE_CAPACITY {
        sender.try_send(value as u8).expect("queue slot");
    }
    assert!(sender.try_send(0).is_err());
    assert_eq!(GATEWAY_MESSAGE_QUEUE_CAPACITY, 256);
}

#[test]
fn sixty_four_message_admissions_are_allowed_and_the_next_is_stable() {
    let work = gateway_work();
    let admissions = (0..MAX_ACTIVE_GATEWAY_MESSAGES)
        .map(|_| work.try_admit_message().expect("message admission"))
        .collect::<Vec<_>>();

    let error = work
        .try_admit_message()
        .expect_err("the sixty-fifth message must be refused");
    assert_eq!(error, GatewayMessageAdmissionError::Busy);
    assert_eq!(error.public_code(), "gateway-busy");
    assert_eq!(work.message_diagnostics().active, 64);
    assert_eq!(work.message_diagnostics().saturation_refusals, 1);

    drop(admissions);
    assert_eq!(work.message_diagnostics().active, 0);

    for _ in 0..(MAX_ACTIVE_GATEWAY_MESSAGES * 4) {
        drop(work.try_admit_message().expect("released slot is reusable"));
    }
    assert_eq!(work.message_diagnostics().active, 0);
}

#[tokio::test]
async fn consumer_and_channel_registries_enforce_their_fixed_capacities() {
    let consumer_work = gateway_work();
    consumer_work
        .spawn_consumer(|_| std::future::pending::<()>())
        .expect("single consumer");
    assert!(consumer_work
        .spawn_consumer(|_| std::future::pending::<()>())
        .is_err());
    assert!(
        consumer_work
            .stop_and_wait(Instant::now() + Duration::from_millis(20))
            .await
    );

    let channel_work = gateway_work();
    for _ in 0..MAX_ACTIVE_GATEWAY_CHANNELS {
        channel_work
            .spawn_channel(|_| std::future::pending::<()>())
            .expect("bounded channel slot");
    }
    assert!(channel_work
        .spawn_channel(|_| std::future::pending::<()>())
        .is_err());
    assert!(
        channel_work
            .stop_and_wait(Instant::now() + Duration::from_millis(20))
            .await
    );
}

#[tokio::test]
async fn stop_closes_the_queue_and_awaits_consumer_three_channels_and_messages() {
    let work = gateway_work();
    let completed = Arc::new(AtomicUsize::new(0));
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<u8>(GATEWAY_MESSAGE_QUEUE_CAPACITY);

    let consumer_done = Arc::clone(&completed);
    work.spawn_consumer(move |cancel| async move {
        tokio::select! {
            _ = cancel.cancelled() => {}
            _ = receiver.recv() => {}
        }
        consumer_done.fetch_add(1, Ordering::SeqCst);
    })
    .expect("consumer starts");

    for _ in 0..3 {
        let channel_done = Arc::clone(&completed);
        work.spawn_channel(move |cancel| async move {
            cancel.cancelled().await;
            channel_done.fetch_add(1, Ordering::SeqCst);
        })
        .expect("simulated channel starts");
    }

    let message_done = Arc::clone(&completed);
    work.spawn_message(move |cancel| async move {
        cancel.cancelled().await;
        message_done.fetch_add(1, Ordering::SeqCst);
    })
    .expect("message starts");

    assert!(
        work.stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert_eq!(completed.load(Ordering::SeqCst), 5);
    assert_eq!(work.consumer_phase(), ServiceWorkPhase::Closed);
    assert_eq!(work.channel_phase(), ServiceWorkPhase::Closed);
    assert_eq!(work.message_phase(), ServiceWorkPhase::Closed);
    assert!(matches!(
        sender.try_send(1),
        Err(tokio::sync::mpsc::error::TrySendError::Closed(1))
    ));
    assert_eq!(
        work.try_admit_message()
            .expect_err("closed run refuses work"),
        GatewayMessageAdmissionError::ShuttingDown
    );
}

#[tokio::test]
async fn a_new_gateway_run_does_not_reuse_closed_registries() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let app_work = coordinator.work_supervisor();
    let stopped = GatewayWorkServices::new(app_work.clone());
    assert!(
        stopped
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );

    let restarted = GatewayWorkServices::new(app_work);
    drop(
        restarted
            .try_admit_message()
            .expect("a new run has fresh fixed slots"),
    );
}
