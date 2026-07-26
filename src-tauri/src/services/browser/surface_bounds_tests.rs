use super::surface_bounds::{
    BrowserSurfaceBounds, NativeSurfaceRect, SurfaceTracker, SurfaceUpdate,
};

fn bounds(generation: u64) -> BrowserSurfaceBounds {
    BrowserSurfaceBounds {
        x: 20,
        y: 40,
        width: 600,
        height: 400,
        visible: true,
        generation,
    }
}

#[test]
fn surface_bounds_reject_invalid_or_excessive_geometry() {
    let mut invalid = bounds(1);
    invalid.x = -1;
    assert!(invalid.validate().is_err());
    invalid = bounds(1);
    invalid.width = 0;
    assert!(invalid.validate().is_err());
    invalid = bounds(1);
    invalid.height = 20_000;
    assert!(invalid.validate().is_err());
}

#[test]
fn surface_tracker_ignores_stale_and_identical_updates() {
    let mut tracker = SurfaceTracker::default();
    let initial = bounds(4);

    assert_eq!(tracker.classify(initial.clone()), SurfaceUpdate::Changed);
    assert_eq!(tracker.classify(initial.clone()), SurfaceUpdate::Stale);
    assert_eq!(tracker.classify(bounds(3)), SurfaceUpdate::Stale);
    assert_eq!(tracker.classify(bounds(5)), SurfaceUpdate::Unchanged);
    let mut changed = bounds(5);
    changed.width = 360;
    changed.generation = 6;
    assert_eq!(tracker.classify(changed), SurfaceUpdate::Changed);
}

#[test]
fn surface_tracker_accepts_a_fresh_generation_after_page_reload() {
    let mut tracker = SurfaceTracker::default();

    assert_eq!(tracker.classify(bounds(9)), SurfaceUpdate::Changed);
    tracker.reset();
    assert_eq!(tracker.classify(bounds(1)), SurfaceUpdate::Changed);
}

#[test]
fn css_coordinates_are_flipped_for_the_native_macos_view() {
    assert_eq!(
        bounds(1).native_rect(900),
        Ok(NativeSurfaceRect {
            x: 20,
            y: 460,
            width: 600,
            height: 400,
        })
    );

    assert!(bounds(1).native_rect(300).is_err());
}

#[test]
fn css_coordinates_scale_for_windows_dpi() {
    for (scale, expected) in [
        (
            1.0,
            NativeSurfaceRect {
                x: 20,
                y: 40,
                width: 600,
                height: 400,
            },
        ),
        (
            1.25,
            NativeSurfaceRect {
                x: 25,
                y: 50,
                width: 750,
                height: 500,
            },
        ),
        (
            1.5,
            NativeSurfaceRect {
                x: 30,
                y: 60,
                width: 900,
                height: 600,
            },
        ),
    ] {
        assert_eq!(bounds(1).scaled_top_left_rect(scale), Ok(expected));
    }
    assert!(bounds(1).scaled_top_left_rect(0.0).is_err());
    assert!(bounds(1).scaled_top_left_rect(f64::NAN).is_err());
}
