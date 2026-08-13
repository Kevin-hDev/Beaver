use super::super::CefUnavailableCategory;
use std::time::Instant;

pub(super) fn ticks_at(deadline: Instant) -> Result<u64, CefUnavailableCategory> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    let remaining =
        u64::try_from(remaining.as_nanos()).map_err(|_| CefUnavailableCategory::Reaper)?;
    now_ticks()?
        .checked_add(remaining)
        .filter(|value| *value != 0)
        .ok_or(CefUnavailableCategory::Reaper)
}

pub(super) fn reached(deadline_ticks: u64) -> Result<bool, CefUnavailableCategory> {
    if deadline_ticks == 0 {
        return Err(CefUnavailableCategory::Reaper);
    }
    Ok(now_ticks()? >= deadline_ticks)
}

pub(super) fn now_ticks() -> Result<u64, CefUnavailableCategory> {
    let mut value = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut value) } != 0
        || value.tv_sec < 0
        || !(0..1_000_000_000).contains(&value.tv_nsec)
    {
        return Err(CefUnavailableCategory::Reaper);
    }
    u64::try_from(value.tv_sec)
        .ok()
        .and_then(|seconds| seconds.checked_mul(1_000_000_000))
        .and_then(|ticks| ticks.checked_add(value.tv_nsec as u64))
        .ok_or(CefUnavailableCategory::Reaper)
}
