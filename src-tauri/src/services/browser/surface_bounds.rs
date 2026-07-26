use serde::{Deserialize, Serialize};

#[cfg(any(test, target_os = "macos", target_os = "windows"))]
const MAX_COORDINATE: i32 = 16_384;
#[cfg(any(test, target_os = "macos", target_os = "windows"))]
const MAX_DIMENSION: u32 = 16_384;
#[cfg(any(test, target_os = "windows"))]
const MIN_SCALE_FACTOR: f64 = 0.5;
#[cfg(any(test, target_os = "windows"))]
const MAX_SCALE_FACTOR: f64 = 8.0;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserSurfaceBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub visible: bool,
    pub generation: u64,
}

#[cfg(any(test, target_os = "macos", target_os = "windows"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NativeSurfaceRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl BrowserSurfaceBounds {
    #[cfg(any(test, target_os = "macos", target_os = "windows"))]
    pub(super) fn validate(&self) -> Result<(), ()> {
        if self.x < 0
            || self.y < 0
            || self.x > MAX_COORDINATE
            || self.y > MAX_COORDINATE
            || self.width == 0
            || self.height == 0
            || self.width > MAX_DIMENSION
            || self.height > MAX_DIMENSION
        {
            return Err(());
        }
        Ok(())
    }

    #[cfg(any(test, target_os = "macos"))]
    pub(super) fn native_rect(&self, parent_height: i32) -> Result<NativeSurfaceRect, ()> {
        self.validate()?;
        let height = i32::try_from(self.height).map_err(|_| ())?;
        let width = i32::try_from(self.width).map_err(|_| ())?;
        let bottom = self.y.checked_add(height).ok_or(())?;
        if parent_height <= 0 || bottom > parent_height {
            return Err(());
        }
        Ok(NativeSurfaceRect {
            x: self.x,
            y: parent_height - bottom,
            width,
            height,
        })
    }

    #[cfg(any(test, target_os = "windows"))]
    pub(super) fn scaled_top_left_rect(&self, scale_factor: f64) -> Result<NativeSurfaceRect, ()> {
        self.validate()?;
        if !scale_factor.is_finite()
            || !(MIN_SCALE_FACTOR..=MAX_SCALE_FACTOR).contains(&scale_factor)
        {
            return Err(());
        }
        Ok(NativeSurfaceRect {
            x: scaled_i32(self.x, scale_factor)?,
            y: scaled_i32(self.y, scale_factor)?,
            width: scaled_i32(i32::try_from(self.width).map_err(|_| ())?, scale_factor)?,
            height: scaled_i32(i32::try_from(self.height).map_err(|_| ())?, scale_factor)?,
        })
    }
}

#[cfg(any(test, target_os = "windows"))]
fn scaled_i32(value: i32, scale_factor: f64) -> Result<i32, ()> {
    let scaled = f64::from(value) * scale_factor;
    if !scaled.is_finite() || scaled < 0.0 || scaled > f64::from(i32::MAX) {
        return Err(());
    }
    Ok(scaled.round() as i32)
}

#[cfg(any(test, target_os = "macos", target_os = "windows"))]
#[derive(Default)]
pub(super) struct SurfaceTracker {
    last: Option<BrowserSurfaceBounds>,
}

#[cfg(any(test, target_os = "macos", target_os = "windows"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SurfaceUpdate {
    Stale,
    Unchanged,
    Changed,
}

#[cfg(any(test, target_os = "macos", target_os = "windows"))]
impl SurfaceTracker {
    pub(super) fn reset(&mut self) {
        self.last = None;
    }

    pub(super) fn classify(&mut self, next: BrowserSurfaceBounds) -> SurfaceUpdate {
        if next.validate().is_err() {
            return SurfaceUpdate::Stale;
        }
        if let Some(previous) = self.last.as_ref() {
            if next.generation <= previous.generation {
                return SurfaceUpdate::Stale;
            }
            let changed = next.x != previous.x
                || next.y != previous.y
                || next.width != previous.width
                || next.height != previous.height
                || next.visible != previous.visible;
            self.last = Some(next);
            return if changed {
                SurfaceUpdate::Changed
            } else {
                SurfaceUpdate::Unchanged
            };
        }
        self.last = Some(next);
        SurfaceUpdate::Changed
    }
}
