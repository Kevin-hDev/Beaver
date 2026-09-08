use super::{browser_slot::BrowserSlot, surface_bounds::BrowserSurfaceBounds};

fn bounds(x: i32, width: u32, generation: u64) -> BrowserSurfaceBounds {
    BrowserSurfaceBounds {
        x,
        y: 180,
        width,
        height: 500,
        visible: true,
        generation,
    }
}

#[test]
fn slot_keeps_the_latest_surface_requested_during_creation() {
    let slot = BrowserSlot::new().expect("runtime epoch available");
    assert!(slot.begin_creation());

    slot.request_surface(&bounds(420, 600, 1))
        .expect("surface request accepted");
    let latest = bounds(560, 460, 2);
    slot.request_surface(&latest)
        .expect("updated surface request accepted");

    assert_eq!(slot.desired_surface(), Some(latest));
}

#[test]
fn closing_a_creating_slot_hides_its_pending_surface() {
    let slot = BrowserSlot::new().expect("runtime epoch available");
    assert!(slot.begin_creation());
    slot.request_surface(&bounds(420, 600, 1))
        .expect("surface request accepted");

    slot.close();

    assert_eq!(
        slot.desired_surface().map(|surface| surface.visible),
        Some(false)
    );
}

#[test]
fn closed_slot_cannot_recreate_favicon_state_via_late_navigation() {
    let slot = BrowserSlot::new().unwrap();
    slot.begin_creation();
    let epoch = slot.live_epoch().unwrap();
    let key = super::browser_view_key::BrowserViewKey {
        session_id: "test".into(),
        tab_id: "tab".into(),
    };
    let mut state = super::favicon_state::FaviconState::default();
    state.begin_document(key.clone(), epoch);
    slot.close();
    state.release_view(&key, epoch);
    slot.mark_closed();
    if let Some(epoch) = slot.live_epoch() {
        state.begin_document(key, epoch);
    }
    assert!(state.entries.is_empty());
    assert_eq!(slot.epoch(), Some(epoch)); // cleanup still has its identity
}
