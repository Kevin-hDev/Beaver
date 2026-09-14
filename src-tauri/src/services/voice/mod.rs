// T02 fixe les contrats que les commandes et moteurs des tâches T05 à T09 consommeront.
#![allow(dead_code)]

pub mod errors;
pub mod limits;
pub mod runtime;
pub mod settings;
pub mod state;
pub mod types;
mod work;

#[cfg(test)]
mod settings_tests;
#[cfg(test)]
mod state_tests;
#[cfg(test)]
mod work_tests;
