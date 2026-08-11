use super::super::CefUnavailableCategory;
use std::time::{Duration, Instant};
use windows_sys::Win32::System::WindowsProgramming::QueryUnbiasedInterruptTimePrecise;

const TICKS_PER_SECOND: u64 = 10_000_000;
const NANOS_PER_TICK: u32 = 100;

pub(super) fn ticks_at(deadline: Instant) -> Result<u64, CefUnavailableCategory> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    let current = monotonic_ticks()?;
    let target = current
        .checked_add(duration_ticks(remaining)?)
        .ok_or(CefUnavailableCategory::Object)?;
    (target != 0)
        .then_some(target)
        .ok_or(CefUnavailableCategory::Object)
}

pub(super) fn reached(deadline_ticks: u64) -> Result<bool, CefUnavailableCategory> {
    if deadline_ticks == 0 {
        return Err(CefUnavailableCategory::Object);
    }
    Ok(monotonic_ticks()? >= deadline_ticks)
}

fn monotonic_ticks() -> Result<u64, CefUnavailableCategory> {
    // Cette horloge exclut les changements de date et partage la même unité de
    // 100 ns entre le parent et ses helpers.
    let mut ticks = 0_u64;
    unsafe { QueryUnbiasedInterruptTimePrecise(&mut ticks) };
    (ticks != 0)
        .then_some(ticks)
        .ok_or(CefUnavailableCategory::Object)
}

fn duration_ticks(duration: Duration) -> Result<u64, CefUnavailableCategory> {
    let seconds = duration
        .as_secs()
        .checked_mul(TICKS_PER_SECOND)
        .ok_or(CefUnavailableCategory::Object)?;
    let subsecond = u64::from(duration.subsec_nanos().div_ceil(NANOS_PER_TICK));
    seconds
        .checked_add(subsecond)
        .ok_or(CefUnavailableCategory::Object)
}
