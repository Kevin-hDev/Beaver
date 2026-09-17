use serde::{Deserialize, Serialize};

use crate::services::reasoning_continuity::contract::ReasoningModeId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(tag = "kind", content = "efforts", rename_all = "snake_case")]
#[cfg_attr(test, ts(tag = "kind", content = "efforts", rename_all = "snake_case"))]
pub enum ReasoningControl {
    Unknown,
    ProviderDefault,
    Toggle,
    Efforts(Vec<ReasoningModeId>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ModelReasoningContract {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub mandatory: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub default_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub supports_max_tokens: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub default_effort: Option<ReasoningModeId>,
    pub control: ReasoningControl,
}

impl ModelReasoningContract {
    #[cfg(test)]
    pub fn from_names(modes: &[&str], default_mode: Option<&str>) -> Option<Self> {
        Self::from_modes(
            true,
            &modes
                .iter()
                .map(|mode| (*mode).to_string())
                .collect::<Vec<_>>(),
            default_mode,
        )
    }

    pub fn from_modes(
        supports_thinking: bool,
        modes: &[String],
        default_mode: Option<&str>,
    ) -> Option<Self> {
        if !supports_thinking {
            return None;
        }
        let parsed = modes
            .iter()
            .map(|mode| ReasoningModeId::from_name(Some(mode)))
            .collect::<Option<Vec<_>>>()?;
        let default_effort = match default_mode {
            Some(mode) => {
                ReasoningModeId::from_name(Some(mode)).filter(|mode| parsed.contains(mode))
            }
            None => None,
        };
        let control = if parsed.is_empty() {
            // Missing controls prove neither a toggle nor mandatory reasoning.
            ReasoningControl::Unknown
        } else {
            // An explicit auto choice remains user-visible. ProviderDefault is reserved
            // for provider metadata that intentionally exposes no control.
            ReasoningControl::Efforts(parsed.clone())
        };
        Some(Self {
            mandatory: (!parsed.is_empty()).then_some(!parsed.contains(&ReasoningModeId::Off)),
            default_enabled: default_effort.map(|mode| mode != ReasoningModeId::Off),
            supports_max_tokens: None,
            default_effort,
            control,
        })
    }

    pub fn selection(&self) -> (Vec<String>, Option<String>) {
        let modes = match &self.control {
            ReasoningControl::Unknown | ReasoningControl::ProviderDefault => {
                vec![ReasoningModeId::Auto]
            }
            ReasoningControl::Toggle => vec![ReasoningModeId::Off, ReasoningModeId::Auto],
            ReasoningControl::Efforts(efforts) => efforts.clone(),
        };
        let modes: Vec<String> = modes
            .into_iter()
            .map(ReasoningModeId::as_name)
            .map(str::to_string)
            .collect();
        let default = self.default_mode_name().map(str::to_string);
        (modes, default)
    }

    pub fn supports_mode(&self, mode: &str) -> bool {
        match &self.control {
            ReasoningControl::Unknown | ReasoningControl::ProviderDefault => mode == "auto",
            ReasoningControl::Toggle => matches!(mode, "off" | "auto"),
            ReasoningControl::Efforts(efforts) => {
                efforts.iter().any(|effort| effort.as_name() == mode)
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        let (modes, _) = self.selection();
        super::provider_model_registry_validation::valid_reasoning_contract(
            true,
            &modes,
            self.default_effort.map(ReasoningModeId::as_name),
        )
        .is_ok()
    }

    pub fn default_mode_name(&self) -> Option<&'static str> {
        if self.default_enabled == Some(false) && self.supports_mode("off") {
            return Some("off");
        }
        if let Some(mode) = self
            .default_effort
            .map(ReasoningModeId::as_name)
            .filter(|mode| self.supports_mode(mode))
        {
            return Some(mode);
        }
        match self.control {
            ReasoningControl::Unknown | ReasoningControl::ProviderDefault => Some("auto"),
            ReasoningControl::Toggle => Some("auto"),
            ReasoningControl::Efforts(_) => None,
        }
    }

    pub const fn is_unknown(&self) -> bool {
        matches!(self.control, ReasoningControl::Unknown)
    }
}

#[cfg(test)]
pub(crate) fn typescript_bindings() -> String {
    use ts_rs::{Config, TS};

    let config = Config::default();
    format!(
        "// @generated from Rust by `cargo test services::llm::model_reasoning_contract_tests::export_typescript_model_reasoning_contract -- --ignored --exact`.\n\
         // Do not edit this file manually.\n\n\
         export {}\n\n\
         export {}\n\n\
         export {}\n",
        ReasoningModeId::decl(&config),
        ReasoningControl::decl(&config),
        ModelReasoningContract::decl(&config),
    )
}
