pub(crate) const MAX_MODEL_ID_BYTES: usize = 128;

pub(crate) fn is_valid_model_id(value: &str) -> bool {
    // OpenRouter documents a single leading `~` for latest aliases (2026-09-09).
    // Validate its suffix without changing the exact ID used by transport/replay.
    let identifier = value.strip_prefix('~').unwrap_or(value);
    !identifier.is_empty()
        && value.len() <= MAX_MODEL_ID_BYTES
        && !identifier.contains("..")
        && !identifier.starts_with('/')
        && identifier.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/' | b':')
        })
}

pub(crate) fn canonical_upstream_owner(owner: &str) -> &str {
    match owner {
        "mistralai" => "mistral",
        "moonshotai" => "moonshot",
        "x-ai" => "xai",
        "z-ai" => "zai",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_id_contract_accepts_routes_and_rejects_unsafe_input() {
        assert!(is_valid_model_id("google/gemma-4-31b-it:free"));
        assert!(is_valid_model_id("plain-model"));
        assert!(!is_valid_model_id("../x"));
        assert!(!is_valid_model_id("/x"));
        assert!(!is_valid_model_id("model\nx"));
        assert!(!is_valid_model_id(&"a".repeat(MAX_MODEL_ID_BYTES + 1)));
    }

    #[test]
    fn upstream_owner_aliases_have_one_canonical_form() {
        assert_eq!(canonical_upstream_owner("mistralai"), "mistral");
        assert_eq!(canonical_upstream_owner("x-ai"), "xai");
        assert_eq!(canonical_upstream_owner("mistral"), "mistral");
    }

    #[test]
    fn latest_alias_prefix_is_valid_without_relaxing_path_or_size_checks() {
        assert!(is_valid_model_id("~openai/gpt-latest"));
        assert!(is_valid_model_id("~anthropic/claude-sonnet-latest"));
        assert!(is_valid_model_id(&format!(
            "~{}",
            "a".repeat(MAX_MODEL_ID_BYTES - 1)
        )));
        for invalid in [
            "~",
            "~~openai/gpt-latest",
            "openai/~gpt-latest",
            "~/x",
            "~../x",
            "~model\nx",
        ] {
            assert!(!is_valid_model_id(invalid), "{invalid:?}");
        }
        assert!(!is_valid_model_id(&format!(
            "~{}",
            "a".repeat(MAX_MODEL_ID_BYTES)
        )));
    }
}
