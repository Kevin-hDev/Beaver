pub(crate) mod benchmarks;
mod cohere;
pub(crate) mod lifecycle;
pub(crate) mod lifecycle_loading;
mod lifecycle_loading_worker;
mod lifecycle_maintenance;
mod lifecycle_release;
mod lifecycle_state;
mod lifecycle_storage;
mod parakeet;
mod qwen;
pub(crate) mod recognizer;
mod vad;

#[cfg(test)]
mod benchmarks_tests;
#[cfg(test)]
mod lifecycle_tests;
#[cfg(test)]
mod recognizer_tests;
