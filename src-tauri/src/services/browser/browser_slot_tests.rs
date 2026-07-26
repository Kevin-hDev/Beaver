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
