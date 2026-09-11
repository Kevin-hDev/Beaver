use crate::models::{AutomationDefinition, AutomationSchedule, AutomationStatus};
use chrono::{DateTime, Duration, LocalResult, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use croner::Cron;
use std::str::FromStr;

const MAX_CRON_EXPRESSION_BYTES: usize = 128;
const MAX_DELAY_MINUTES: u32 = 525_600;
const MAX_DST_GAP_MINUTES: usize = 24 * 60;
const MISSED_GRACE_MINUTES: i64 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleError {
    InvalidSchedule,
    CalculationFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NextFire {
    pub at: DateTime<Utc>,
    pub dst_adjusted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DueRange {
    pub first: DateTime<Utc>,
    pub last: DateTime<Utc>,
    pub count: u64,
    pub dst_adjusted_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DueBatch {
    pub missed: Option<DueRange>,
    pub admissible: Option<DueRange>,
}

pub fn next_fire_at(
    definition: &AutomationDefinition,
    after: DateTime<Utc>,
) -> Result<Option<NextFire>, ScheduleError> {
    if definition.status != AutomationStatus::Active {
        return Ok(None);
    }
    let next = match &definition.schedule {
        AutomationSchedule::Once {
            local_datetime,
            timezone,
        } => resolve_local(*timezone, *local_datetime)?,
        AutomationSchedule::Cron {
            expression,
            timezone,
        } => next_cron(expression, *timezone, after)?,
        AutomationSchedule::AfterCompletion { delay_minutes } => {
            if !(1..=MAX_DELAY_MINUTES).contains(delay_minutes) {
                return Err(ScheduleError::InvalidSchedule);
            }
            let anchor = definition.anchor_at.ok_or(ScheduleError::InvalidSchedule)?;
            let at = anchor
                .checked_add_signed(Duration::minutes(i64::from(*delay_minutes)))
                .ok_or(ScheduleError::CalculationFailed)?;
            NextFire {
                at,
                dst_adjusted: false,
            }
        }
    };
    Ok((next.at > after).then_some(next))
}

pub fn due_between(
    definition: &AutomationDefinition,
    checked_after: DateTime<Utc>,
    through: DateTime<Utc>,
) -> Result<DueBatch, ScheduleError> {
    if definition.status != AutomationStatus::Active || through <= checked_after {
        return Ok(DueBatch::default());
    }
    let mut batch = DueBatch::default();
    let threshold = through - Duration::minutes(MISSED_GRACE_MINUTES);
    let mut cursor = checked_after;
    while let Some(next) = next_fire_at(definition, cursor)? {
        if next.at > through {
            break;
        }
        let range = if next.at < threshold {
            &mut batch.missed
        } else {
            &mut batch.admissible
        };
        extend_range(range, next)?;
        cursor = next.at;
        if !matches!(definition.schedule, AutomationSchedule::Cron { .. }) {
            break;
        }
    }
    Ok(batch)
}

fn extend_range(range: &mut Option<DueRange>, next: NextFire) -> Result<(), ScheduleError> {
    match range {
        Some(existing) => {
            existing.last = next.at;
            existing.count = existing
                .count
                .checked_add(1)
                .ok_or(ScheduleError::CalculationFailed)?;
            if next.dst_adjusted {
                existing.dst_adjusted_count = existing
                    .dst_adjusted_count
                    .checked_add(1)
                    .ok_or(ScheduleError::CalculationFailed)?;
            }
        }
        None => {
            *range = Some(DueRange {
                first: next.at,
                last: next.at,
                count: 1,
                dst_adjusted_count: u64::from(next.dst_adjusted),
            });
        }
    }
    Ok(())
}

fn next_cron(
    expression: &str,
    timezone: Tz,
    after: DateTime<Utc>,
) -> Result<NextFire, ScheduleError> {
    let cron = parse_cron(expression)?;
    let local_after = after.with_timezone(&timezone);
    let next = cron
        .find_next_occurrence(&local_after, false)
        .map_err(|_| ScheduleError::CalculationFailed)?;
    let dst_adjusted = !cron
        .is_time_matching(&next)
        .map_err(|_| ScheduleError::CalculationFailed)?;
    Ok(NextFire {
        at: next.with_timezone(&Utc),
        dst_adjusted,
    })
}

fn parse_cron(expression: &str) -> Result<Cron, ScheduleError> {
    if expression.is_empty()
        || expression.len() > MAX_CRON_EXPRESSION_BYTES
        || expression.split_whitespace().count() != 5
        || expression
            .chars()
            .any(|character| !matches!(character, '0'..='9' | '*' | ',' | '-' | '/' | ' '))
    {
        return Err(ScheduleError::InvalidSchedule);
    }
    Cron::from_str(expression).map_err(|_| ScheduleError::InvalidSchedule)
}

fn resolve_local(timezone: Tz, local_datetime: NaiveDateTime) -> Result<NextFire, ScheduleError> {
    match timezone.from_local_datetime(&local_datetime) {
        LocalResult::Single(at) => Ok(NextFire {
            at: at.with_timezone(&Utc),
            dst_adjusted: false,
        }),
        LocalResult::Ambiguous(first, second) => Ok(NextFire {
            at: first.min(second).with_timezone(&Utc),
            dst_adjusted: false,
        }),
        LocalResult::None => resolve_gap(timezone, local_datetime),
    }
}

fn resolve_gap(timezone: Tz, start: NaiveDateTime) -> Result<NextFire, ScheduleError> {
    let mut candidate = start;
    for _ in 0..MAX_DST_GAP_MINUTES {
        candidate = candidate
            .checked_add_signed(Duration::minutes(1))
            .ok_or(ScheduleError::CalculationFailed)?;
        match timezone.from_local_datetime(&candidate) {
            LocalResult::Single(at) => {
                return Ok(NextFire {
                    at: at.with_timezone(&Utc),
                    dst_adjusted: true,
                });
            }
            LocalResult::Ambiguous(first, second) => {
                return Ok(NextFire {
                    at: first.min(second).with_timezone(&Utc),
                    dst_adjusted: true,
                });
            }
            LocalResult::None => {}
        }
    }
    Err(ScheduleError::CalculationFailed)
}
