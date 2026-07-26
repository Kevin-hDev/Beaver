use super::types::DatedValue;
use crate::services::forecast::input_dates::count_frequency_steps;
use crate::services::forecast::limits::MAX_PREDICTIONS;
use std::collections::HashSet;

#[derive(Default)]
pub struct SequenceFindings {
    pub duplicates: usize,
    pub unordered: usize,
    pub missing: usize,
    pub inconsistent: usize,
    pub invalid_future: usize,
    pub samples: Vec<String>,
}

pub fn inspect(history: &[DatedValue], future: &[DatedValue], frequency: &str) -> SequenceFindings {
    let mut findings = SequenceFindings::default();
    inspect_order(history, frequency, &mut findings, false);
    inspect_order(future, frequency, &mut findings, true);

    if let (Some(last), Some(first)) = (history.last(), future.first()) {
        if count_frequency_steps(last.date, first.date, frequency, MAX_PREDICTIONS) != Some(1) {
            findings.invalid_future += 1;
            push_sample(&mut findings, &first.raw);
        }
    }
    findings
}

fn inspect_order(
    points: &[DatedValue],
    frequency: &str,
    findings: &mut SequenceFindings,
    future: bool,
) {
    let mut seen = HashSet::with_capacity(points.len());
    for point in points {
        if !seen.insert(point.date) {
            findings.duplicates += 1;
            push_sample(findings, &point.raw);
        }
    }
    for pair in points.windows(2) {
        if pair[1].date <= pair[0].date {
            findings.unordered += 1;
            push_sample(findings, &pair[1].raw);
            continue;
        }
        match count_frequency_steps(pair[0].date, pair[1].date, frequency, MAX_PREDICTIONS) {
            Some(1) => {}
            Some(steps) if future => findings.invalid_future += steps.saturating_sub(1),
            Some(steps) => findings.missing += steps.saturating_sub(1),
            None if future => findings.invalid_future += 1,
            None => findings.inconsistent += 1,
        }
    }
}

fn push_sample(findings: &mut SequenceFindings, value: &str) {
    if findings.samples.len() < crate::services::forecast::limits::MAX_ISSUE_SAMPLES {
        findings.samples.push(value.chars().take(80).collect());
    }
}
