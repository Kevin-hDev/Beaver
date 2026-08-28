use super::agent_local::types_ollama::OllamaThink;
use super::reasoning_continuity::contract::ReasoningModeId;

const MAX_CAPABILITIES: usize = 32;
const MAX_CAPABILITY_BYTES: usize = 64;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EffectiveOllamaReasoning {
    pub mode: ReasoningModeId,
    pub mode_name: String,
    pub payload: OllamaThink,
}

pub(crate) fn supported_modes(model: &str) -> Vec<String> {
    let modes = if uses_effort_wire(model) {
        &["low", "medium", "high"][..]
    } else {
        &["off", "auto"][..]
    };
    modes.iter().map(|mode| (*mode).to_string()).collect()
}

pub(crate) fn resolve(
    model: &str,
    requested_mode: Option<&str>,
    thinking_enabled: bool,
    capabilities: Option<&[String]>,
) -> Result<EffectiveOllamaReasoning, ()> {
    let capabilities = capabilities.ok_or(())?;
    if capabilities.is_empty()
        || capabilities.len() > MAX_CAPABILITIES
        || capabilities
            .iter()
            .any(|capability| capability.len() > MAX_CAPABILITY_BYTES)
    {
        return Err(());
    }
    let supports_thinking = capabilities.iter().any(|value| value == "thinking");
    if !supports_thinking || !thinking_enabled {
        return Ok(EffectiveOllamaReasoning {
            mode: ReasoningModeId::Off,
            mode_name: "off".to_string(),
            payload: OllamaThink::Bool(false),
        });
    }
    let modes = supported_modes(model);
    let mode_name = requested_mode
        .filter(|requested| modes.iter().any(|mode| mode == requested))
        .map(str::to_string)
        .unwrap_or_else(|| {
            if uses_effort_wire(model) {
                "medium".to_string()
            } else {
                "auto".to_string()
            }
        });
    let mode = ReasoningModeId::from_name(Some(&mode_name)).ok_or(())?;
    let payload = payload(model, Some(&mode_name), true);
    Ok(EffectiveOllamaReasoning {
        mode,
        mode_name,
        payload,
    })
}

pub(crate) fn payload(model: &str, mode: Option<&str>, fallback: bool) -> OllamaThink {
    if uses_effort_wire(model) {
        let effort = match mode {
            Some("low" | "medium" | "high") => mode.unwrap(),
            Some("xhigh") => "high",
            _ => "medium",
        };
        return OllamaThink::Level(effort.to_string());
    }
    OllamaThink::Bool(super::reasoning::enabled(mode, fallback))
}

// Ollama attend une chaîne d'effort pour cette famille et un booléen pour les autres.
fn uses_effort_wire(model: &str) -> bool {
    model.to_lowercase().contains("gpt-oss")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_drives_exact_mode_and_payload_for_supported_families() {
        let cases = [
            (
                "gpt-oss:20b",
                ReasoningModeId::Medium,
                OllamaThink::Level("medium".into()),
            ),
            ("qwen3.5:4b", ReasoningModeId::Auto, OllamaThink::Bool(true)),
            (
                "deepseek-r1:latest",
                ReasoningModeId::Auto,
                OllamaThink::Bool(true),
            ),
            (
                "gemma4:e2b-it-q4_K_M",
                ReasoningModeId::Auto,
                OllamaThink::Bool(true),
            ),
        ];
        for (model, mode, payload) in cases {
            let effective = resolve(model, None, true, Some(&["thinking".into()])).unwrap();
            assert_eq!(effective.mode, mode, "{model}");
            assert_eq!(effective.payload, payload, "{model}");
        }
        let disabled =
            resolve("llama3.2:latest", None, true, Some(&["completion".into()])).unwrap();
        assert_eq!(disabled.mode, ReasoningModeId::Off);
        assert_eq!(disabled.payload, OllamaThink::Bool(false));
        assert!(resolve("qwen3.5:4b", None, true, None).is_err());
        assert!(resolve("qwen3.5:4b", None, true, Some(&[])).is_err());
    }
}
