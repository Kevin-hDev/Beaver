use super::live_session_registry::LiveSessionRegistry;

#[test]
fn first_activation_is_cold_and_following_activations_are_live() {
    let mut registry = LiveSessionRegistry::default();
    assert!(registry.activate("first"));
    assert!(!registry.activate("first"));
}

#[test]
fn live_session_registry_is_bounded() {
    let mut registry = LiveSessionRegistry::default();
    for index in 0..65 {
        assert!(registry.activate(&format!("session-{index}")));
    }
    assert_eq!(registry.len(), 64);
    assert!(registry.activate("session-0"));
}
