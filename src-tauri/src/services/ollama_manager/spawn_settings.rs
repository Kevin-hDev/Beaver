use super::compute_mode::OllamaComputeMode;
use std::ffi::OsString;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OllamaSpawnSettings {
    compute_mode: OllamaComputeMode,
    multi_model: bool,
}

impl OllamaSpawnSettings {
    pub(crate) fn from_config(hardware_accel: &str, multi_model: bool) -> Self {
        Self {
            compute_mode: OllamaComputeMode::from_setting(hardware_accel),
            multi_model,
        }
    }

    pub(crate) fn compute_mode(self) -> OllamaComputeMode {
        self.compute_mode
    }

    pub(crate) fn environment_overrides(self) -> Vec<(OsString, OsString)> {
        let mut overrides = self.compute_mode.environment_overrides();
        // Ollama attend la libération du runner actif avant de charger le suivant
        // quand cette limite vaut 1. Zéro rend l'autorité à son mode automatique.
        overrides.push((
            OsString::from("OLLAMA_MAX_LOADED_MODELS"),
            OsString::from(if self.multi_model { "0" } else { "1" }),
        ));
        overrides
    }
}
