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
    for (index, id) in ["first", "second"].into_iter().enumerate() {
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
        router.update_host_activity(
            &super::host_identity::HostIdentity::ThirdParty(id.to_string()),
            1,
            super::types::ExtensionEventActivity {
                queued: index as u64 + 1,
                delivered: 1,
                dropped: 1,
                timed_out: 1,
                active_handlers: 1,
            },
        );
        router.update_host_activity(
            &super::host_identity::HostIdentity::ThirdParty(id.to_string()),
            2,
            super::types::ExtensionEventActivity {
                queued: 99,
                ..Default::default()
            },
        );
    }

    let activity = router.activity();
    assert_eq!(activity.queued, 3);
    assert_eq!(activity.delivered, 2);
    assert_eq!(activity.dropped, 4);
    assert_eq!(activity.timed_out, 2);
    assert_eq!(activity.active_handlers, 2);
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
    assert_eq!(
        super::event_delivery::host_queue_acknowledgement(&serde_json::json!({"queued": true})),
        Some(true)
    );
    assert_eq!(
        super::event_delivery::host_queue_acknowledgement(&serde_json::json!({"queued": false})),
        Some(false)
    );
    assert_eq!(
        super::event_delivery::host_queue_acknowledgement(&serde_json::json!({"activity": {}})),
        None
    );
}

#[test]
fn host_event_request_matches_the_host_dispatch_shape() {
    let envelope = super::event_payload::EventEnvelope::build(
        super::event_payload::turn_started("session", "request"),
        7,
    )
    .unwrap();
    let request = super::event_delivery::host_event_request(envelope).unwrap();

    assert_eq!(request["event"], "session.turn.started");
    assert_eq!(request["payload"]["type"], "session.turn.started");
    assert_eq!(request["payload"]["sequence"], 7);
    assert_eq!(request["payload"]["sessionId"], "session");
    assert_eq!(request["payload"]["payload"]["status"], "started");
}
