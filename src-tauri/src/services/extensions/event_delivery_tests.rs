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
fn activity_aggregates_bounded_delivery_counters() {
    use std::collections::BTreeSet;

    let router = super::event_delivery::EventRouter::default();
    for id in ["first", "second"] {
        let (delivery, _receiver) = super::event_delivery::EventDelivery::test_delivery();
        let envelope = super::event_payload::EventEnvelope::build(
            super::event_payload::turn_started("session", id),
            1,
        )
        .unwrap();
        delivery.enqueue(envelope.clone());
        delivery.enqueue(envelope);
        router.install(
            super::host_identity::HostIdentity::ThirdParty(id.to_string()),
            1,
            BTreeSet::from(["session.turn.started".to_string()]),
            delivery,
        );
    }

    let activity = router.activity();
    assert_eq!(activity.queued, 2);
    assert_eq!(activity.dropped, 2);
    assert_eq!(activity.delivered, 0);
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

#[test]
fn terminal_event_clears_a_flow_after_the_last_delivery_is_removed() {
    use std::collections::BTreeSet;

    let router = super::event_delivery::EventRouter::default();
    let identity = super::host_identity::HostIdentity::ThirdParty("observer".to_string());
    let (delivery, _receiver) = super::event_delivery::EventDelivery::test_delivery();
    router.install(
        identity.clone(),
        1,
        BTreeSet::from(["session.turn.started".to_string()]),
        delivery,
    );
    assert!(router.publish(super::event_payload::turn_started("session", "request")));
    router.clear(&identity);
    assert!(!router.publish(super::event_payload::turn_terminal(
        "session",
        "request",
        "session.turn.completed",
        "completed",
    )));

    let (replacement, _receiver) = super::event_delivery::EventDelivery::test_delivery();
    router.install(
        identity,
        2,
        BTreeSet::from(["session.turn.started".to_string()]),
        replacement,
    );
    assert!(router.publish(super::event_payload::turn_started("session", "request")));
}

#[test]
fn host_queue_rejection_is_not_a_delivery() {
    assert!(super::event_delivery::host_enqueued(
        &serde_json::json!({"queued": true})
    ));
    assert!(!super::event_delivery::host_enqueued(
        &serde_json::json!({"queued": false})
    ));
    assert!(!super::event_delivery::host_enqueued(
        &serde_json::json!({"activity": {}})
    ));
}
