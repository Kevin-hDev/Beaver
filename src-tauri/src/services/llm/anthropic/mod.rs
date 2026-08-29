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
mod stream;
#[allow(
    dead_code,
    reason = "continuation blocks are consumed by the next activation step"
)]
mod stream_state;
mod stream_state_support;
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
#[allow(
    unused_imports,
    reason = "candidate stream consumer is wired into dispatch after live validation"
)]
pub(super) use stream::consume_stream;

#[cfg(test)]
mod models_tests;
#[cfg(test)]
mod payload_tests;
#[cfg(test)]
mod stream_tests;
