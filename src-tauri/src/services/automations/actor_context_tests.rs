use super::actor_context::*;
use super::{AutomationActor, AutomationError, AutomationOrigin};
use uuid::Uuid;

static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn actor(id: &str) -> AutomationActor {
    AutomationActor {
        origin: AutomationOrigin::Session,
        session_or_channel_id: id.into(),
        current_automation_id: Some(Uuid::new_v4()),
    }
}

#[test]
fn registered_actor_is_trusted_only_for_the_request_lifetime() {
    let _test = TEST_LOCK.lock().unwrap();
    let request_id = Uuid::new_v4().to_string();
    let expected = actor("session-a");
    let guard = register_actor(&request_id, expected.clone()).unwrap();
    assert!(is_automation_request(&request_id));
    assert_eq!(
        actor_for("forged", false, None, Some(&request_id))
            .unwrap()
            .current_automation_id,
        expected.current_automation_id
    );
    drop(guard);
    assert!(!is_automation_request(&request_id));
    assert_eq!(
        actor_for("session-a", false, None, Some(&request_id))
            .unwrap()
            .current_automation_id,
        None
    );
}

#[test]
fn gateway_identity_is_hashed_and_desktop_identity_is_preserved() {
    let _test = TEST_LOCK.lock().unwrap();
    let gateway = actor_for("session-g", true, Some("secret-channel-key"), None).unwrap();
    assert_eq!(gateway.origin, AutomationOrigin::ExternalChannel);
    assert!(gateway.session_or_channel_id.starts_with("gateway:"));
    assert!(!gateway.session_or_channel_id.contains("secret-channel-key"));
    let desktop = actor_for("session-a", false, None, None).unwrap();
    assert_eq!(desktop.origin, AutomationOrigin::Session);
    assert_eq!(desktop.session_or_channel_id, "session-a");
}

#[test]
fn registry_is_bounded_and_guards_release_entries() {
    let _test = TEST_LOCK.lock().unwrap();
    let guards = (0..MAX_ACTORS)
        .map(|index| register_actor(&format!("request-{index}"), actor("session-a")).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        register_actor("overflow", actor("session-a")).unwrap_err(),
        AutomationError::CapacityReached
    );
    drop(guards);
    assert!(register_actor("available", actor("session-a")).is_ok());
}
