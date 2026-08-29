mod client;
#[allow(
    dead_code,
    reason = "candidate Messages transport is activated after live validation"
)]
pub(super) mod messages;
pub(super) mod models;
#[allow(
    dead_code,
    reason = "candidate Messages transport is activated after live validation"
)]
mod payload;
#[allow(
    dead_code,
    reason = "candidate Messages transport is activated after live validation"
)]
pub(super) mod tools;

pub(super) use client::{list_models, test_connection};
#[allow(
    unused_imports,
    reason = "candidate transport payload is consumed when dispatch is activated"
)]
pub(super) use payload::{build_payload, BuildError, PreparedPayload};

#[cfg(test)]
mod models_tests;
#[cfg(test)]
mod payload_tests;
