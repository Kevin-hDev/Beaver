use crate::models::{ScheduledWakeup, WakeupSchedule};
use chrono::{DateTime, Duration, Local};

use super::next_fire::{latest_fire_between, next_fire_at};

pub const MISSED_GRACE_MIN: i64 = 5;

pub fn due_wakeups_at(
    wakeups: &[ScheduledWakeup],
    now: DateTime<Local>,
    target: DateTime<Local>,
) -> Vec<ScheduledWakeup> {
    wakeups
        .iter()
        .filter(|w| w.active && !w.paused_by_global)
        .filter(|w| next_fire_at(&w.schedule, now) == Some(target))
        .cloned()
        .collect()
}

pub fn missed_occurrences(
    wakeups: &[ScheduledWakeup],
    last_checked: DateTime<Local>,
    now: DateTime<Local>,
) -> Vec<(ScheduledWakeup, DateTime<Local>)> {
    let cutoff = now - Duration::minutes(MISSED_GRACE_MIN);
    wakeups
        .iter()
        .filter(|w| w.active && !w.paused_by_global)
        .filter_map(|w| {
            latest_fire_between(&w.schedule, last_checked, cutoff).map(|dt| (w.clone(), dt))
        })
        .collect()
}

pub fn is_late(target: DateTime<Local>, now: DateTime<Local>) -> bool {
    now - target > Duration::minutes(MISSED_GRACE_MIN)
}

pub fn is_once(wakeup: &ScheduledWakeup) -> bool {
    matches!(wakeup.schedule, WakeupSchedule::Once { .. })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn local(y: i32, m: u32, d: u32, h: u32, mi: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(y, m, d, h, mi, 0).single().unwrap()
    }

    fn wakeup(id: &str, time: &str) -> ScheduledWakeup {
        ScheduledWakeup {
            id: id.into(),
            name: id.into(),
            model: "m".into(),
            provider: "ollama".into(),
            prompt: "p".into(),
            schedule: WakeupSchedule::Daily { time: time.into() },
            description: String::new(),
            active: true,
            paused_by_global: false,
            created_at: "2026-05-17T00:00:00Z".into(),
        }
    }

    #[test]
    fn selects_all_wakeups_due_at_same_minute() {
        let now = local(2026, 5, 17, 7, 0);
        let target = local(2026, 5, 17, 8, 0);
        let due = due_wakeups_at(&[wakeup("a", "08:00"), wakeup("b", "08:00")], now, target);
        assert_eq!(due.len(), 2);
    }

    #[test]
    fn misses_only_after_grace_period() {
        let target = local(2026, 5, 17, 8, 0);
        assert!(!is_late(target, local(2026, 5, 17, 8, 5)));
        assert!(is_late(target, local(2026, 5, 17, 8, 6)));
    }

    #[test]
    fn finds_missed_occurrence_after_last_check() {
        let missed = missed_occurrences(
            &[wakeup("a", "08:00")],
            local(2026, 5, 17, 7, 0),
            local(2026, 5, 17, 8, 10),
        );
        assert_eq!(missed.len(), 1);
        assert_eq!(missed[0].1, local(2026, 5, 17, 8, 0));
    }
}
