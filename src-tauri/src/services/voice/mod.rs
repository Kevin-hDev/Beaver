// T02 fixe les contrats que les commandes et moteurs des tâches T05 à T09 consommeront.
#![allow(dead_code)]

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
pub mod download;
pub mod errors;
pub mod limits;
#[cfg(any(target_os = "macos", windows))]
pub mod maintenance;
#[cfg(any(target_os = "macos", windows))]
pub mod model;
mod nullable;
#[cfg(any(target_os = "macos", windows))]
pub mod pipeline;
#[cfg(any(target_os = "macos", windows))]
pub mod recovery;
pub mod runtime;
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
