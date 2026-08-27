use super::agent_local::types_ollama::OllamaThink;
use super::reasoning_continuity::contract::ReasoningModeId;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EffectiveReasoningProfile {
    pub mode: ReasoningModeId,
    pub mode_name: Option<String>,
    pub active: bool,
    pub supports_thinking: bool,
    pub ollama_payload: Option<OllamaThink>,
}

impl EffectiveReasoningProfile {
    pub(crate) fn api(
        provider: &str,
        model: &str,
        requested_mode: Option<&str>,
        thinking_enabled: bool,
        supports_thinking: bool,
    ) -> Result<Self, ()> {
        if !supports_thinking {
            return Ok(Self::off(false, None));
        }
        let requested = if thinking_enabled {
            requested_mode
        } else {
            Some("off")
        };
        let mode_name =
            super::reasoning::normalize_for_model(provider, model, requested, supports_thinking)
                .ok_or(())?;
        let mode = ReasoningModeId::from_name(Some(&mode_name)).ok_or(())?;
        Ok(Self {
            mode,
            active: mode != ReasoningModeId::Off,
            mode_name: Some(mode_name),
            supports_thinking,
            ollama_payload: None,
        })
    }

    pub(crate) fn ollama(
        model: &str,
        requested_mode: Option<&str>,
        thinking_enabled: bool,
        capabilities: Option<&[String]>,
    ) -> Result<Self, ()> {
        let effective = super::reasoning_ollama::resolve(
            model,
            requested_mode,
            thinking_enabled,
            capabilities,
        )?;
        let supports_thinking =
            capabilities.is_some_and(|values| values.iter().any(|value| value == "thinking"));
        Ok(Self {
            mode: effective.mode,
            mode_name: supports_thinking.then_some(effective.mode_name),
            active: effective.payload.enabled(),
            supports_thinking,
            ollama_payload: Some(effective.payload),
        })
    }

    fn off(supports_thinking: bool, payload: Option<OllamaThink>) -> Self {
        Self {
            mode: ReasoningModeId::Off,
            mode_name: None,
            active: false,
            supports_thinking,
            ollama_payload: payload,
        }
    }
}
