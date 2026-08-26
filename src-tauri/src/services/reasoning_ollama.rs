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
    let mode_name =
        super::reasoning::normalize_for_model("ollama", model, requested_mode, true).ok_or(())?;
    let mode = ReasoningModeId::from_name(Some(&mode_name)).ok_or(())?;
    let payload = super::reasoning::ollama_think(model, Some(&mode_name), true).ok_or(())?;
    Ok(EffectiveOllamaReasoning {
        mode,
        mode_name,
        payload,
    })
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
