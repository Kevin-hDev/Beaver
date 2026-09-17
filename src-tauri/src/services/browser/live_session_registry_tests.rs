use super::{live_session_registry::LiveSessionRegistry, session_model::SessionModel};

fn id(index: usize) -> String {
    format!("{index:032x}")
}

#[test]
fn inserted_sessions_are_reused() {
    let mut registry = LiveSessionRegistry::default();
    registry.insert("first".into(), SessionModel::new(id(1)).unwrap());

    assert!(registry.contains("first"));
    assert_eq!(registry.get_mut("first").unwrap().state().tabs[0].id, id(1));
}

#[test]
fn live_session_registry_is_bounded() {
    let mut registry = LiveSessionRegistry::default();
    for index in 0..65 {
        registry.insert(
            format!("session-{index}"),
            SessionModel::new(id(index + 1)).unwrap(),
        );
    }
    assert_eq!(registry.len(), 64);
    assert!(!registry.contains("session-0"));
    assert!(registry.contains("session-64"));
}
