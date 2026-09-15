#[cfg(any(target_os = "macos", windows))]
pub mod actions;
#[cfg(any(target_os = "macos", windows))]
pub mod assembly;
pub mod audio_buffer;
#[cfg(any(target_os = "macos", windows))]
pub mod capture;
pub mod contracts;
#[cfg(any(target_os = "macos", windows))]
pub mod delivery;
#[cfg(any(target_os = "macos", windows))]
mod delivery_types;
pub mod download;
pub mod errors;
mod language;
pub mod limits;
#[cfg(any(target_os = "macos", windows))]
pub mod maintenance;
#[cfg(any(target_os = "macos", windows))]
pub mod model;
mod nullable;
#[cfg(any(target_os = "macos", windows))]
mod operation;
#[cfg(any(target_os = "macos", windows))]
pub mod pipeline;
#[cfg(any(target_os = "macos", windows))]
mod pipeline_benchmark;
#[cfg(any(target_os = "macos", windows))]
mod pipeline_stop;
#[cfg(any(target_os = "macos", windows))]
pub mod recovery;
pub mod runtime;
#[cfg(test)]
mod runtime_test_support;
pub mod settings;
pub mod slicing;
#[cfg(any(target_os = "macos", windows))]
pub mod start_guards;
pub mod state;
#[cfg(any(target_os = "macos", windows))]
pub mod transcription;
pub mod types;
mod work;

#[cfg(all(test, any(target_os = "macos", windows)))]
mod actions_tests;
#[cfg(all(test, any(target_os = "macos", windows)))]
mod delivery_tests;
#[cfg(all(test, any(target_os = "macos", windows)))]
mod maintenance_tests;
#[cfg(all(test, any(target_os = "macos", windows)))]
mod pipeline_tests;
#[cfg(all(test, any(target_os = "macos", windows)))]
mod recovery_tests;
#[cfg(test)]
mod settings_tests;
#[cfg(all(test, any(target_os = "macos", windows)))]
mod start_guards_tests;
#[cfg(test)]
mod state_tests;
#[cfg(test)]
mod work_tests;

#[cfg(all(test, any(target_os = "macos", windows)))]
mod native_contract_tests;
#[cfg(all(test, any(target_os = "macos", windows)))]
mod prototype_tests;
