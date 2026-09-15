// T02 fixe les contrats que les commandes et moteurs des tâches T05 à T09 consommeront.
#![allow(dead_code)]

#[cfg(any(target_os = "macos", windows))]
pub mod assembly;
pub mod audio_buffer;
#[cfg(any(target_os = "macos", windows))]
pub mod capture;
pub mod download;
pub mod errors;
pub mod limits;
#[cfg(any(target_os = "macos", windows))]
pub mod model;
pub mod runtime;
pub mod settings;
pub mod slicing;
pub mod state;
#[cfg(any(target_os = "macos", windows))]
pub mod transcription;
pub mod types;
mod work;

#[cfg(test)]
mod settings_tests;
#[cfg(test)]
mod state_tests;
#[cfg(test)]
mod work_tests;

#[cfg(all(test, any(target_os = "macos", windows)))]
mod native_contract_tests;
#[cfg(all(test, any(target_os = "macos", windows)))]
mod prototype_tests;
