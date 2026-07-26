use super::next_fire::{latest_fire_between, next_fire_at};
use crate::models::WakeupSchedule;
use chrono::{DateTime, Local, TimeZone};

fn local(y: i32, m: u32, d: u32, h: u32, mi: u32) -> DateTime<Local> {
    Local
        .with_ymd_and_hms(y, m, d, h, mi, 0)
        .single()
        .expect("valid datetime")
}

#[test]
fn once_future_ok() {
    let now = local(2026, 4, 16, 8, 0);
    let s = WakeupSchedule::Once {
        datetime: "2026-04-16T08:30".into(),
    };
    assert_eq!(next_fire_at(&s, now), Some(local(2026, 4, 16, 8, 30)));
}

#[test]
fn once_past_none() {
    let now = local(2026, 4, 16, 9, 0);
    let s = WakeupSchedule::Once {
        datetime: "2026-04-16T08:30".into(),
    };
    assert_eq!(next_fire_at(&s, now), None);
}

#[test]
fn daily_today_if_future() {
    let now = local(2026, 4, 16, 7, 0);
    let s = WakeupSchedule::Daily {
        time: "08:00".into(),
    };
    assert_eq!(next_fire_at(&s, now), Some(local(2026, 4, 16, 8, 0)));
}

#[test]
fn daily_tomorrow_if_past() {
    let now = local(2026, 4, 16, 9, 0);
    let s = WakeupSchedule::Daily {
        time: "08:00".into(),
    };
    assert_eq!(next_fire_at(&s, now), Some(local(2026, 4, 17, 8, 0)));
}

#[test]
fn weekly_same_day_future() {
    let now = local(2026, 4, 16, 7, 0);
    let s = WakeupSchedule::Weekly {
        weekday: 3,
        time: "08:00".into(),
    };
    assert_eq!(next_fire_at(&s, now), Some(local(2026, 4, 16, 8, 0)));
}

#[test]
fn weekly_next_week_if_passed() {
    let now = local(2026, 4, 16, 9, 0);
    let s = WakeupSchedule::Weekly {
        weekday: 3,
        time: "08:00".into(),
    };
    assert_eq!(next_fire_at(&s, now), Some(local(2026, 4, 23, 8, 0)));
}

#[test]
fn latest_daily_between_finds_missed_today() {
    let after = local(2026, 4, 16, 7, 0);
    let before = local(2026, 4, 16, 9, 0);
    let s = WakeupSchedule::Daily {
        time: "08:00".into(),
    };
    assert_eq!(
        latest_fire_between(&s, after, before),
        Some(local(2026, 4, 16, 8, 0))
    );
}

#[test]
fn latest_weekly_between_finds_missed_day() {
    let after = local(2026, 4, 15, 7, 0);
    let before = local(2026, 4, 16, 9, 0);
    let s = WakeupSchedule::Weekly {
        weekday: 3,
        time: "08:00".into(),
    };
    assert_eq!(
        latest_fire_between(&s, after, before),
        Some(local(2026, 4, 16, 8, 0))
    );
}
