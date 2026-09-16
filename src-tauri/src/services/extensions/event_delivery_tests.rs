#[test]
fn one_terminal_event_per_admitted_agent_turn() {
    let router = super::event_delivery::EventRouter::default();
    let (delivery, _receiver) = super::event_delivery::EventDelivery::test_delivery();
    router.install(
        super::host_identity::HostIdentity::ThirdParty("observer".to_string()),
        1,
        std::collections::BTreeSet::from(["session.turn.started".to_string()]),
        delivery,
    );
    assert!(router.publish(super::event_payload::turn_started("session", "request")));
    assert!(router.publish(super::event_payload::turn_terminal(
        "session",
        "request",
        "session.turn.completed",
        "completed",
    )));
    assert!(!router.publish(super::event_payload::turn_terminal(
        "session",
        "request",
        "session.turn.failed",
        "failed",
    )));
}

#[test]
fn slow_observer_does_not_block_or_grow_unbounded() {
    let (delivery, _receiver) = super::event_delivery::EventDelivery::test_delivery();
    let envelope = super::event_payload::EventEnvelope::build(
        super::event_payload::turn_started("session", "request"),
        1,
    )
    .unwrap();
    delivery.enqueue(envelope.clone());
    delivery.enqueue(envelope);
    assert_eq!(delivery.dropped(), 1);
}

#[test]
fn stopping_generation_is_not_a_failed_neighbor() {
    use std::collections::BTreeSet;

    let router = super::event_delivery::EventRouter::default();
    let (stopped, _stopped_receiver) = super::event_delivery::EventDelivery::test_delivery();
    let (neighbor, _neighbor_receiver) = super::event_delivery::EventDelivery::test_delivery();
    let stopped_probe = stopped.clone();
    let neighbor_probe = neighbor.clone();
    let subscriptions = BTreeSet::from(["session.turn.started".to_string()]);
    router.install(
        super::host_identity::HostIdentity::ThirdParty("stopped".to_string()),
        1,
        subscriptions.clone(),
        stopped,
    );
    router.install(
        super::host_identity::HostIdentity::ThirdParty("neighbor".to_string()),
        1,
        subscriptions.clone(),
        neighbor,
    );
    let (replacement, _replacement_receiver) =
        super::event_delivery::EventDelivery::test_delivery();
    router.install(
        super::host_identity::HostIdentity::ThirdParty("stopped".to_string()),
        2,
        subscriptions,
        replacement,
    );

    assert!(stopped_probe.is_cancelled());
    assert!(!neighbor_probe.is_cancelled());
}
