use std::ffi::OsString;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OllamaComputeMode {
    Cpu,
    Gpu,
}

impl OllamaComputeMode {
    pub(crate) fn from_setting(value: &str) -> Self {
        if value == "cpu" {
            Self::Cpu
        } else {
            Self::Gpu
        }
    }

    pub(crate) fn environment_overrides(self) -> Vec<(OsString, OsString)> {
        let library = match self {
            // Ollama documente cette variable comme l'autorité qui court-circuite
            // l'autodétection ; le mode CPU doit donc franchir la frontière de spawn.
            Self::Cpu => "cpu",
            // Une valeur vide rend l'autorité à l'autodétection d'Ollama et neutralise
            // un ancien forçage CPU hérité du processus parent.
            Self::Gpu => "",
        };
        vec![(
            OsString::from("OLLAMA_LLM_LIBRARY"),
            OsString::from(library),
        )]
    }
}
