use super::next_fire::{due_between, next_fire_at, parse_timezone, ScheduleError};
use crate::models::{AutomationDefinition, AutomationSchedule, AutomationStatus, AutomationTarget};
use chrono::{NaiveDate, TimeZone, Utc};
use uuid::Uuid;

fn utc(y: i32, m: u32, d: u32, h: u32, minute: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, h, minute, 0)
        .single()
        .expect("valid UTC datetime")
}
fn local(y: i32, m: u32, d: u32, h: u32, minute: u32) -> chrono::NaiveDateTime {
    NaiveDate::from_ymd_opt(y, m, d)
        .expect("valid date")
        .and_hms_opt(h, minute, 0)
        .expect("valid time")
}
fn definition(schedule: AutomationSchedule) -> AutomationDefinition {
    AutomationDefinition {
        id: Uuid::nil(),
        revision: 1,
        name: "test".into(),
        description: None,
        prompt: "run".into(),
        creator_session_id: Some("creator".into()),
        target: AutomationTarget::NewSession { project_id: None },
        provider: "openai".into(),
        model: "gpt-test".into(),
        schedule,
        status: AutomationStatus::Active,
        created_at: utc(2026, 1, 1, 0, 0),
        anchor_at: None,
    }
}

#[test]
fn once_uses_its_persisted_timezone() {
    let automation = definition(AutomationSchedule::Once {
        local_datetime: local(2026, 9, 10, 18, 30),
        timezone: chrono_tz::Europe::Paris,
    });
    let next = next_fire_at(&automation, utc(2026, 9, 10, 16, 0))
        .expect("valid schedule")
        .expect("future occurrence");
    assert_eq!(next.at, utc(2026, 9, 10, 16, 30));
    assert!(!next.dst_adjusted);
}

#[test]
fn once_in_the_past_has_no_next_occurrence() {
    let automation = definition(AutomationSchedule::Once {
        local_datetime: local(2026, 9, 10, 18, 30),
        timezone: chrono_tz::Europe::Paris,
    });
    assert_eq!(
        next_fire_at(&automation, utc(2026, 9, 10, 16, 30)),
        Ok(None)
    );
}

#[test]
fn cron_every_minute_is_strictly_after_the_cursor() {
    let automation = definition(AutomationSchedule::Cron {
        expression: "* * * * *".into(),
        timezone: chrono_tz::UTC,
    });
    let next = next_fire_at(&automation, utc(2026, 9, 10, 12, 0))
        .expect("valid schedule")
        .expect("next occurrence");
    assert_eq!(next.at, utc(2026, 9, 10, 12, 1));
}

#[test]
fn cron_restricted_day_of_month_or_weekday_uses_unix_or_semantics() {
    let automation = definition(AutomationSchedule::Cron {
        expression: "0 9 13 * 0".into(),
        timezone: chrono_tz::UTC,
    });
    let by_month_day = next_fire_at(&automation, utc(2026, 5, 11, 10, 0))
        .unwrap()
        .unwrap();
    let by_weekday = next_fire_at(&automation, utc(2026, 5, 13, 10, 0))
        .unwrap()
        .unwrap();
    assert_eq!(by_month_day.at, utc(2026, 5, 13, 9, 0));
    assert_eq!(by_weekday.at, utc(2026, 5, 17, 9, 0));
}

#[test]
fn after_completion_uses_only_the_persisted_anchor() {
    let mut automation = definition(AutomationSchedule::AfterCompletion { delay_minutes: 10 });
    automation.anchor_at = Some(utc(2026, 9, 10, 12, 0));
    let next = next_fire_at(&automation, utc(2026, 9, 10, 12, 4))
        .unwrap()
        .unwrap();
    assert_eq!(next.at, utc(2026, 9, 10, 12, 10));
}

#[test]
fn inactive_automations_have_no_next_occurrence() {
    for status in [AutomationStatus::Disabled, AutomationStatus::Completed] {
        let mut automation = definition(AutomationSchedule::Cron {
            expression: "* * * * *".into(),
            timezone: chrono_tz::UTC,
        });
        automation.status = status;
        assert_eq!(next_fire_at(&automation, utc(2026, 9, 10, 12, 0)), Ok(None));
    }
}

#[test]
fn cron_rejects_noncanonical_grammar() {
    for expression in ["0 * * * * *", "0 9 * JAN *", "@daily"] {
        let automation = definition(AutomationSchedule::Cron {
            expression: expression.into(),
            timezone: chrono_tz::UTC,
        });
        assert_eq!(
            next_fire_at(&automation, utc(2026, 9, 10, 12, 0)),
            Err(ScheduleError::InvalidSchedule),
            "expression {expression:?} must be rejected"
        );
    }
}

#[test]
fn invalid_iana_timezone_is_rejected() {
    assert_eq!(
        parse_timezone("Mars/Olympus"),
        Err(ScheduleError::InvalidTimezone)
    );
}

#[test]
fn after_completion_delay_is_bounded() {
    for delay_minutes in [0, 525_601] {
        let mut automation = definition(AutomationSchedule::AfterCompletion { delay_minutes });
        automation.anchor_at = Some(utc(2026, 9, 10, 12, 0));
        assert_eq!(
            next_fire_at(&automation, utc(2026, 9, 10, 12, 0)),
            Err(ScheduleError::InvalidSchedule)
        );
    }
}

#[test]
fn nonexistent_spring_minute_moves_to_the_first_valid_minute() {
    let automation = definition(AutomationSchedule::Once {
        local_datetime: local(2026, 3, 29, 2, 30),
        timezone: chrono_tz::Europe::Paris,
    });
    let next = next_fire_at(&automation, utc(2026, 3, 28, 0, 0))
        .unwrap()
        .unwrap();
    assert_eq!(next.at, utc(2026, 3, 29, 1, 0));
    assert!(next.dst_adjusted);
}

#[test]
fn repeated_autumn_minute_keeps_only_the_first_occurrence() {
    let automation = definition(AutomationSchedule::Once {
        local_datetime: local(2026, 10, 25, 2, 30),
        timezone: chrono_tz::Europe::Paris,
    });
    let first = next_fire_at(&automation, utc(2026, 10, 24, 0, 0))
        .unwrap()
        .unwrap();
    assert_eq!(first.at, utc(2026, 10, 25, 0, 30));
    assert_eq!(
        next_fire_at(&automation, utc(2026, 10, 25, 0, 30)),
        Ok(None)
    );
}

#[test]
fn due_batch_aggregates_old_occurrences_and_bounds_the_grace_window() {
    let automation = definition(AutomationSchedule::Cron {
        expression: "* * * * *".into(),
        timezone: chrono_tz::UTC,
    });
    let batch = due_between(&automation, utc(2026, 9, 10, 8, 50), utc(2026, 9, 10, 9, 0)).unwrap();
    let missed = batch.missed.unwrap();
    assert_eq!(
        (missed.first, missed.last, missed.count),
        (utc(2026, 9, 10, 8, 51), utc(2026, 9, 10, 8, 54), 4)
    );
    let admissible = batch.admissible.unwrap();
    assert_eq!(
        (admissible.first, admissible.last, admissible.count),
        (utc(2026, 9, 10, 8, 55), utc(2026, 9, 10, 9, 0), 6)
    );
}

#[test]
fn persisted_timezone_does_not_depend_on_the_current_system_timezone() {
    let automation = definition(AutomationSchedule::Once {
        local_datetime: local(2026, 12, 1, 9, 0),
        timezone: "Asia/Tokyo".parse().expect("known IANA timezone"),
    });
    let next = next_fire_at(&automation, utc(2026, 11, 30, 0, 0))
        .unwrap()
        .unwrap();
    assert_eq!(next.at, utc(2026, 12, 1, 0, 0));
}
