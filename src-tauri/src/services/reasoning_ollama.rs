use super::agent_local::types_ollama::OllamaThink;
use super::reasoning_continuity::contract::ReasoningModeId;

const MAX_CAPABILITIES: usize = 32;
const MAX_CAPABILITY_BYTES: usize = 64;
const GPT_OSS_MODES: &[&str] = &["low", "medium", "high"];
const GLM_FLASH_CLOUD_MODES: &[&str] = &["low", "high", "max"];

#[derive(Debug, Clone, Copy)]
struct EffortProfile {
    modes: &'static [&'static str],
    default_mode: &'static str,
    mandatory: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EffectiveOllamaReasoning {
    pub mode: ReasoningModeId,
    pub mode_name: String,
    pub payload: OllamaThink,
}

pub(crate) fn supported_modes(model: &str) -> Vec<String> {
    let modes = effort_profile(model)
        .map(|profile| profile.modes)
        .unwrap_or(&["off", "auto"]);
    modes.iter().map(|mode| (*mode).to_string()).collect()
}

pub(crate) fn default_mode(model: &str) -> &'static str {
    effort_profile(model)
        .map(|profile| profile.default_mode)
        .unwrap_or("auto")
}

pub(crate) fn requires_thinking(model: &str) -> bool {
    effort_profile(model).is_some_and(|profile| profile.mandatory)
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
    let profile = effort_profile(model);
    // A mandatory profile cannot silently become a non-thinking model when
    // the daemon reports inconsistent metadata. Admission must stop instead.
    if !supports_thinking && profile.is_some_and(|value| value.mandatory) {
        return Err(());
    }
    if !supports_thinking || (!thinking_enabled && !profile.is_some_and(|value| value.mandatory)) {
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
        .unwrap_or_else(|| default_mode(model).to_string());
    let mode = ReasoningModeId::from_name(Some(&mode_name)).ok_or(())?;
    let payload = payload(model, Some(&mode_name), true);
    Ok(EffectiveOllamaReasoning {
        mode,
        mode_name,
        payload,
    })
}

pub(crate) fn payload(model: &str, mode: Option<&str>, fallback: bool) -> OllamaThink {
    if let Some(profile) = effort_profile(model) {
        let effort = match mode {
            Some(requested) if profile.modes.contains(&requested) => requested,
            Some("xhigh") if !profile.mandatory && profile.modes.contains(&"high") => "high",
            _ => profile.default_mode,
        };
        return OllamaThink::Level(effort.to_string());
    }
    OllamaThink::Bool(super::reasoning::enabled(mode, fallback))
}

// Le profil local est l'autorité unique des modes, du défaut et du caractère obligatoire.
fn effort_profile(model: &str) -> Option<EffortProfile> {
    if model.eq_ignore_ascii_case("glm-5.3-flash:cloud") {
        return Some(EffortProfile {
            modes: GLM_FLASH_CLOUD_MODES,
            // Beaver explicitly selects the upstream documented default;
            // the Ollama cloud page confirms this effort is accepted.
            default_mode: "max",
            mandatory: true,
        });
    }
    if model.to_lowercase().contains("gpt-oss") {
        return Some(EffortProfile {
            modes: GPT_OSS_MODES,
            default_mode: "medium",
            mandatory: false,
        });
    }
    None
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

    #[test]
    fn glm_flash_cloud_uses_exact_efforts_even_for_legacy_off() {
        let model = "glm-5.3-flash:cloud";
        assert_eq!(supported_modes(model), ["low", "high", "max"]);
        for mode in ["low", "high", "max"] {
            let resolved = resolve(model, Some(mode), true, Some(&["thinking".into()])).unwrap();
            assert_eq!(resolved.payload, OllamaThink::Level(mode.into()));
        }
        let legacy = resolve(model, Some("off"), false, Some(&["thinking".into()])).unwrap();
        assert!(legacy.payload.enabled());
    }

    #[test]
    fn mandatory_cloud_profile_rejects_inconsistent_show_capabilities() {
        for capabilities in [None, Some(Vec::new()), Some(vec!["completion".into()])] {
            assert!(resolve(
                "glm-5.3-flash:cloud",
                Some("max"),
                true,
                capabilities.as_deref()
            )
            .is_err());
        }
    }
}
