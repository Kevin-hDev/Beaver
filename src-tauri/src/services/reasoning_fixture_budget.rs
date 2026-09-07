use std::cell::RefCell;
use std::future::Future;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub(crate) const TURN_TIMEOUT: Duration = Duration::from_secs(240);

const DEFAULT_OUTPUT_TOKENS: usize = 512;
const MAX_OUTPUT_TOKENS: usize = 8_192;
const DEFAULT_ATTEMPTS: usize = 4;
const MAX_ATTEMPTS: usize = 4;
const DEFAULT_INPUT_BYTES: usize = 8_192;
const MAX_INPUT_BYTES: usize = 65_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FixtureLimits {
    pub(crate) output_tokens: u32,
    pub(crate) attempts: u32,
    pub(crate) input_bytes: usize,
}

impl FixtureLimits {
    pub(crate) fn from_env() -> Result<Self, String> {
        Self::from_values(
            read_env("BEAVER_FIXTURE_OUTPUT_TOKENS")?.as_deref(),
            read_env("BEAVER_FIXTURE_ATTEMPTS")?.as_deref(),
            read_env("BEAVER_FIXTURE_INPUT_BYTES")?.as_deref(),
        )
    }

    pub(crate) fn from_values(
        output_tokens: Option<&str>,
        attempts: Option<&str>,
        input_bytes: Option<&str>,
    ) -> Result<Self, String> {
        Ok(Self {
            output_tokens: bounded(output_tokens, DEFAULT_OUTPUT_TOKENS, MAX_OUTPUT_TOKENS)? as u32,
            attempts: bounded(attempts, DEFAULT_ATTEMPTS, MAX_ATTEMPTS)? as u32,
            input_bytes: bounded(input_bytes, DEFAULT_INPUT_BYTES, MAX_INPUT_BYTES)?,
        })
    }
}

fn read_env(name: &str) -> Result<Option<String>, String> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(_) => Err("fixture limits invalid".into()),
    }
}

fn bounded(value: Option<&str>, default: usize, maximum: usize) -> Result<usize, String> {
    let Some(value) = value else {
        return Ok(default);
    };
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "fixture limits invalid")?;
    (parsed > 0 && parsed <= maximum)
        .then_some(parsed)
        .ok_or_else(|| "fixture limits invalid".to_string())
}

struct FixtureBudgetState {
    limits: FixtureLimits,
    remaining_attempts: u32,
}

tokio::task_local! {
    static BUDGET: RefCell<FixtureBudgetState>;
}

pub(crate) async fn run_scoped<F, T>(
    limits: FixtureLimits,
    cancel: CancellationToken,
    future: F,
) -> Result<T, String>
where
    F: Future<Output = Result<T, String>>,
{
    let state = FixtureBudgetState {
        remaining_attempts: limits.attempts,
        limits,
    };
    // Drop the actual pending request, not merely its ActiveStreams entry.
    tokio::select! {
        biased;
        _ = cancel.cancelled() => Err("Annulé".into()),
        _ = tokio::time::sleep(TURN_TIMEOUT) => {
            cancel.cancel();
            Err("fixture timeout".into())
        },
        result = BUDGET.scope(RefCell::new(state), future) => result,
    }
}

pub(crate) fn output_limit(existing: Option<u32>) -> Option<u32> {
    BUDGET
        .try_with(|budget| {
            let limit = budget.borrow().limits.output_tokens;
            Some(match existing {
                Some(value) if value > 0 => value.min(limit),
                _ => limit,
            })
        })
        .unwrap_or(existing)
}

pub(crate) fn is_active() -> bool {
    BUDGET.try_with(|_| true).unwrap_or(false)
}

pub(crate) fn authorize_payload(payload: &impl serde::Serialize) -> Result<(), String> {
    let Ok(maximum) = BUDGET.try_with(|budget| budget.borrow().limits.input_bytes) else {
        return Ok(());
    };
    // Count while serializing: oversized opaque history needs no duplicate buffer.
    let mut counter = ByteCounter {
        written: 0,
        maximum,
    };
    serde_json::to_writer(&mut counter, payload)
        .map_err(|_| "fixture input limit exceeded".to_string())?;
    authorize_serialized_len(counter.written)
}

struct ByteCounter {
    written: usize,
    maximum: usize,
}

impl std::io::Write for ByteCounter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.maximum.saturating_sub(self.written) {
            return Err(std::io::Error::other("fixture input limit exceeded"));
        }
        self.written += bytes.len();
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) fn authorize_serialized_len(length: usize) -> Result<(), String> {
    BUDGET
        .try_with(|budget| {
            let mut budget = budget.borrow_mut();
            if length > budget.limits.input_bytes {
                return Err("fixture input limit exceeded".to_string());
            }
            if budget.remaining_attempts == 0 {
                return Err("fixture attempt limit exceeded".to_string());
            }
            budget.remaining_attempts -= 1;
            Ok(())
        })
        .unwrap_or(Ok(()))
}

#[cfg(test)]
#[path = "reasoning_fixture_budget_tests.rs"]
mod tests;
