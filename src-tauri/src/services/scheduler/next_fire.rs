use crate::models::WakeupSchedule;
use chrono::{DateTime, Datelike, Duration, Local, NaiveDateTime, NaiveTime, TimeZone, Weekday};

/// Renvoie la prochaine date/heure de déclenchement strictement future.
/// `None` si :
/// - Once dont la date est passée
/// - parsing échoue
pub fn next_fire_at(schedule: &WakeupSchedule, now: DateTime<Local>) -> Option<DateTime<Local>> {
    match schedule {
        WakeupSchedule::Once { datetime } => parse_once(datetime, now),
        WakeupSchedule::Daily { time } => parse_daily(time, now),
        WakeupSchedule::Weekly { weekday, time } => parse_weekly(*weekday, time, now),
    }
}

pub fn latest_fire_between(
    schedule: &WakeupSchedule,
    after: DateTime<Local>,
    before: DateTime<Local>,
) -> Option<DateTime<Local>> {
    if before <= after {
        return None;
    }
    match schedule {
        WakeupSchedule::Once { datetime } => {
            let dt = parse_once_raw(datetime)?;
            (dt > after && dt <= before).then_some(dt)
        }
        WakeupSchedule::Daily { time } => latest_daily_between(time, after, before),
        WakeupSchedule::Weekly { weekday, time } => {
            latest_weekly_between(*weekday, time, after, before)
        }
    }
}

fn parse_once(s: &str, now: DateTime<Local>) -> Option<DateTime<Local>> {
    let dt = parse_once_raw(s)?;
    if dt > now {
        Some(dt)
    } else {
        None
    }
}

fn parse_once_raw(s: &str) -> Option<DateTime<Local>> {
    let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").ok()?;
    Local.from_local_datetime(&naive).single()
}

fn parse_time_hhmm(s: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(s, "%H:%M").ok()
}

fn parse_daily(time: &str, now: DateTime<Local>) -> Option<DateTime<Local>> {
    let nt = parse_time_hhmm(time)?;
    let today_at = Local
        .from_local_datetime(&now.date_naive().and_time(nt))
        .single()?;
    if today_at > now {
        Some(today_at)
    } else {
        let tomorrow = (now.date_naive() + Duration::days(1)).and_time(nt);
        Local.from_local_datetime(&tomorrow).single()
    }
}

fn parse_weekly(weekday_idx: u8, time: &str, now: DateTime<Local>) -> Option<DateTime<Local>> {
    let target_wd = weekday_from_idx(weekday_idx)?;
    let nt = parse_time_hhmm(time)?;

    let today_idx = now.weekday().num_days_from_monday() as i64;
    let target_idx = target_wd.num_days_from_monday() as i64;
    let mut delta = target_idx - today_idx;
    if delta < 0 {
        delta += 7;
    }

    let candidate = (now.date_naive() + Duration::days(delta)).and_time(nt);
    let dt = Local.from_local_datetime(&candidate).single()?;
    if dt > now {
        Some(dt)
    } else {
        // Même jour mais heure passée → semaine suivante
        let next_week = (now.date_naive() + Duration::days(delta + 7)).and_time(nt);
        Local.from_local_datetime(&next_week).single()
    }
}

fn latest_daily_between(
    time: &str,
    after: DateTime<Local>,
    before: DateTime<Local>,
) -> Option<DateTime<Local>> {
    let nt = parse_time_hhmm(time)?;
    let today = Local
        .from_local_datetime(&before.date_naive().and_time(nt))
        .single()?;
    if today <= before && today > after {
        return Some(today);
    }
    let yesterday = (before.date_naive() - Duration::days(1)).and_time(nt);
    let dt = Local.from_local_datetime(&yesterday).single()?;
    (dt > after && dt <= before).then_some(dt)
}

fn latest_weekly_between(
    weekday_idx: u8,
    time: &str,
    after: DateTime<Local>,
    before: DateTime<Local>,
) -> Option<DateTime<Local>> {
    let target = weekday_from_idx(weekday_idx)?;
    let nt = parse_time_hhmm(time)?;
    let before_idx = before.weekday().num_days_from_monday() as i64;
    let target_idx = target.num_days_from_monday() as i64;
    let mut delta = before_idx - target_idx;
    if delta < 0 {
        delta += 7;
    }
    let candidate = (before.date_naive() - Duration::days(delta)).and_time(nt);
    let dt = Local.from_local_datetime(&candidate).single()?;
    (dt > after && dt <= before).then_some(dt)
}

fn weekday_from_idx(idx: u8) -> Option<Weekday> {
    match idx {
        0 => Some(Weekday::Mon),
        1 => Some(Weekday::Tue),
        2 => Some(Weekday::Wed),
        3 => Some(Weekday::Thu),
        4 => Some(Weekday::Fri),
        5 => Some(Weekday::Sat),
        6 => Some(Weekday::Sun),
        _ => None,
    }
}
