use std::time::{Duration, Instant};

use super::types_stream::StreamResult;

const MIN_LIVE_INTERVAL_NS: u64 = 100_000_000;
const MAX_GENERATION_DURATION_NS: u64 = 7 * 24 * 60 * 60 * 1_000_000_000;

#[derive(Debug, Default)]
pub struct GenerationTracker {
    first_activity: Option<Instant>,
    last_activity: Option<Instant>,
    first_counted_tokens: Option<u32>,
    native_duration_ns: Option<u64>,
}

impl GenerationTracker {
    pub fn start_activity(&mut self) -> bool {
        self.start_activity_at(Instant::now())
    }

    pub fn record_activity(&mut self, token_count: u32) -> bool {
        self.record_activity_at(token_count, Instant::now())
    }

    pub fn record_native_duration(&mut self, duration_ns: u64) {
        self.native_duration_ns = valid_duration_ns(duration_ns).then_some(duration_ns);
    }

    pub fn live_tps(&self, token_count: u32) -> f64 {
        let Some(duration_ns) = self.observed_duration_ns() else {
            return 0.0;
        };
        if duration_ns < MIN_LIVE_INTERVAL_NS {
            return 0.0;
        }
        let generated = token_count.saturating_sub(self.first_counted_tokens.unwrap_or(token_count));
        rate(generated as u64, duration_ns)
    }

    fn sample(&self, exact_tokens: Option<u32>, estimated_tokens: u32) -> Option<GenerationSample> {
        let (duration_ns, native) = match self.native_duration_ns {
            Some(duration_ns) => (duration_ns, true),
            None => (self.observed_duration_ns()?, false),
        };
        let tokens = if native {
            exact_tokens.unwrap_or(estimated_tokens)
        } else {
            estimated_tokens.saturating_sub(
                self.first_counted_tokens.unwrap_or(estimated_tokens),
            )
        };
        (tokens > 0).then_some(GenerationSample {
            tokens: tokens as u64,
            duration_ns,
            estimated: !native || exact_tokens.is_none(),
        })
    }

    fn observed_duration(&self) -> Option<Duration> {
        self.last_activity?.checked_duration_since(self.first_activity?)
    }

    fn observed_duration_ns(&self) -> Option<u64> {
        let duration_ns = u64::try_from(self.observed_duration()?.as_nanos()).ok()?;
        valid_duration_ns(duration_ns).then_some(duration_ns)
    }

    fn start_activity_at(&mut self, now: Instant) -> bool {
        let first = self.first_activity.is_none();
        self.first_activity.get_or_insert(now);
        self.last_activity = Some(now);
        first
    }

    fn record_activity_at(&mut self, token_count: u32, now: Instant) -> bool {
        let first = self.start_activity_at(now);
        self.first_counted_tokens.get_or_insert(token_count);
        first
    }
}

#[derive(Debug, Default)]
pub struct GenerationAggregate {
    tokens: u64,
    duration_ns: u64,
    samples: u32,
    estimated: bool,
}

impl GenerationAggregate {
    pub fn add_result(&mut self, result: &StreamResult) {
        let Some(sample) = result
            .generation
            .sample(result.eval_count, result.estimated_output_tokens())
        else {
            self.estimated |= result
                .eval_count
                .unwrap_or_else(|| result.estimated_output_tokens())
                > 0;
            return;
        };
        self.tokens = self.tokens.saturating_add(sample.tokens);
        self.duration_ns = self.duration_ns.saturating_add(sample.duration_ns);
        self.samples = self.samples.saturating_add(1);
        self.estimated |= sample.estimated;
    }

    pub fn merge(&mut self, other: Self) {
        self.estimated |= other.estimated;
        if other.samples == 0 {
            return;
        }
        self.tokens = self.tokens.saturating_add(other.tokens);
        self.duration_ns = self.duration_ns.saturating_add(other.duration_ns);
        self.samples = self.samples.saturating_add(other.samples);
    }

    pub fn summary(&self) -> GenerationSummary {
        GenerationSummary {
            duration_ns: self.duration_ns,
            tps: rate(self.tokens, self.duration_ns),
            estimated: self.samples == 0 || self.estimated,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GenerationSummary {
    pub duration_ns: u64,
    pub tps: f64,
    pub estimated: bool,
}

#[derive(Debug)]
struct GenerationSample {
    tokens: u64,
    duration_ns: u64,
    estimated: bool,
}

pub fn valid_duration_ns(duration_ns: u64) -> bool {
    duration_ns > 0 && duration_ns <= MAX_GENERATION_DURATION_NS
}

pub fn seconds_to_duration_ns(seconds: f64) -> Option<u64> {
    if !seconds.is_finite() || seconds <= 0.0 {
        return None;
    }
    let duration_ns = seconds * 1_000_000_000.0;
    if duration_ns > u64::MAX as f64 {
        return None;
    }
    let duration_ns = duration_ns.round() as u64;
    valid_duration_ns(duration_ns).then_some(duration_ns)
}

fn rate(tokens: u64, duration_ns: u64) -> f64 {
    if tokens == 0 || duration_ns == 0 {
        return 0.0;
    }
    tokens as f64 / (duration_ns as f64 / 1_000_000_000.0)
}

#[cfg(test)]
#[path = "generation_metrics_tests.rs"]
mod tests;
