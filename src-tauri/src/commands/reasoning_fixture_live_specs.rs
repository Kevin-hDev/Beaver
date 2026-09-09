pub(crate) struct LiveSpec {
    pub(crate) provider: &'static str,
    pub(crate) model: &'static str,
    pub(crate) region: &'static str,
    pub(crate) mode: &'static str,
    pub(crate) report_variant: bool,
    pub(crate) vision: bool,
}

const fn spec(
    provider: &'static str,
    model: &'static str,
    region: &'static str,
    mode: &'static str,
) -> LiveSpec {
    LiveSpec {
        provider,
        model,
        region,
        mode,
        report_variant: false,
        vision: false,
    }
}

const fn variant_spec(
    provider: &'static str,
    model: &'static str,
    region: &'static str,
    mode: &'static str,
) -> LiveSpec {
    LiveSpec {
        provider,
        model,
        region,
        mode,
        report_variant: true,
        vision: true,
    }
}

const fn vision_spec(
    provider: &'static str,
    model: &'static str,
    region: &'static str,
    mode: &'static str,
) -> LiveSpec {
    LiveSpec {
        provider,
        model,
        region,
        mode,
        report_variant: false,
        vision: true,
    }
}

pub(crate) const LIVE_SPECS: &[LiveSpec] = &[
    spec("ollama", "gemma4:e2b-it-q4_K_M", "local", "auto"),
    spec("ollama", "qwen3.5:4b", "local", "auto"),
    spec("google", "gemini-3.5-flash", "france", "medium"),
    spec("mistral", "mistral-small-2603", "france", "high"),
    spec("openrouter", "moonshotai/kimi-k2.5", "france", "medium"),
    // The current toggle uses auto; keep its proof separate from historical medium.
    LiveSpec {
        report_variant: true,
        ..spec("openrouter", "moonshotai/kimi-k2.5", "france", "auto")
    },
    spec("openai", "gpt-5.6-luna", "france", "medium"),
    spec("deepseek", "deepseek-v4-flash", "france", "low"),
    spec("deepseek", "deepseek-v4-flash", "france", "high"),
    spec("deepseek", "deepseek-v4-flash", "france", "max"),
    spec("xai", "grok-4.6", "france", "high"),
    spec("xai-oauth", "grok-4.6", "local", "high"),
    spec("moonshot", "kimi-k2.7-code", "france", "auto"),
    spec("zai", "glm-4.5-flash", "local", "auto"),
    spec("codex-oauth", "gpt-5.6-luna", "local", "medium"),
    spec("cerebras", "gpt-oss-120b", "france", "high"),
    spec("anthropic", "claude-haiku-4-5-20251001", "france", "low"),
    vision_spec("anthropic", "claude-haiku-4-5-20251001", "france", "medium"),
    spec("anthropic", "claude-haiku-4-5-20251001", "france", "high"),
    spec("qwen", "qwen3.8-flash", "singapore", "low"),
    vision_spec("qwen", "qwen3.8-flash", "singapore", "medium"),
    spec("qwen", "qwen3.8-flash", "singapore", "xhigh"),
    variant_spec("google", "gemini-3.8-flash", "france", "low"),
    variant_spec("google", "gemini-3.8-flash", "france", "medium"),
    variant_spec("google", "gemini-3.8-flash", "france", "high"),
    variant_spec("zai", "glm-5.3-flash", "france", "low"),
    variant_spec("zai", "glm-5.3-flash", "france", "high"),
    variant_spec("zai", "glm-5.3-flash", "france", "max"),
    variant_spec("openai", "gpt-6-astra", "france", "low"),
    variant_spec("openai", "gpt-6-astra", "france", "medium"),
    variant_spec("openai", "gpt-6-astra", "france", "high"),
    variant_spec("openai", "gpt-6-astra", "france", "xhigh"),
    variant_spec("openai", "gpt-6-astra", "france", "max"),
    variant_spec("openrouter", "google/gemini-3.8-flash", "france", "low"),
    variant_spec("openrouter", "google/gemini-3.8-flash", "france", "medium"),
    variant_spec("openrouter", "google/gemini-3.8-flash", "france", "high"),
    variant_spec("openrouter", "z-ai/glm-5.3-flash", "france", "low"),
    variant_spec("openrouter", "z-ai/glm-5.3-flash", "france", "high"),
    variant_spec("openrouter", "z-ai/glm-5.3-flash", "france", "max"),
    variant_spec("openrouter", "openai/gpt-6-astra", "france", "low"),
    variant_spec("openrouter", "openai/gpt-6-astra", "france", "medium"),
    variant_spec("openrouter", "openai/gpt-6-astra", "france", "high"),
    variant_spec("openrouter", "openai/gpt-6-astra", "france", "xhigh"),
    variant_spec("openrouter", "openai/gpt-6-astra", "france", "max"),
    variant_spec("ollama", "glm-5.3-flash:cloud", "local", "low"),
    variant_spec("ollama", "glm-5.3-flash:cloud", "local", "high"),
    variant_spec("ollama", "glm-5.3-flash:cloud", "local", "max"),
    variant_spec("codex-oauth", "gpt-6-astra", "local", "low"),
    variant_spec("codex-oauth", "gpt-6-astra", "local", "medium"),
    variant_spec("codex-oauth", "gpt-6-astra", "local", "high"),
    variant_spec("codex-oauth", "gpt-6-astra", "local", "xhigh"),
    variant_spec("codex-oauth", "gpt-6-astra", "local", "max"),
    variant_spec("codex-oauth", "gpt-6-astra", "local", "ultra"),
];
