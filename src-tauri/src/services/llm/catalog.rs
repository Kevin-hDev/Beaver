//! Catalogue statique des providers LLM free-tier supportés.
//!
//! Chaque `ProviderSpec` contient tout ce dont on a besoin pour :
//! - Afficher la card dans le modal "Connecteurs API"
//! - Construire un `OpenAiCompatProvider` à partir d'un `provider_id` + clé

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ProviderSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub category: &'static str, // "llm"
    pub base_url: &'static str,
    pub models_endpoint: &'static str, // chemin relatif à base_url
    pub signup_url: &'static str,
    pub free_tier_label: &'static str,
    pub short_description: &'static str,
    pub short_description_en: &'static str,
    /// Certains providers plafonnent la sortie si `max_tokens` absent
    /// (OpenAI/DeepSeek = 4k, Gemini = 8k). On force un max raisonnable.
    /// Groq/Mistral/Cerebras = unbounded → None (pas de plafond).
    /// **Groq** : surtout ne PAS mettre car leur free tier compte max_tokens dans le TPM budget.
    pub default_max_tokens: Option<u32>,
}

/// Retourne la spec d'un provider LLM par son id.
pub fn find(provider_id: &str) -> Option<&'static ProviderSpec> {
    LLM_PROVIDERS.iter().find(|p| p.id == provider_id)
}

pub const LLM_PROVIDERS: &[ProviderSpec] = &[
    ProviderSpec {
        id: "groq",
        display_name: "Groq",
        category: "llm",
        base_url: "https://api.groq.com/openai/v1",
        models_endpoint: "/models",
        signup_url: "https://console.groq.com/keys",
        free_tier_label: "14 400 req/day",
        short_description: "Inférence ultra-rapide Llama / Mixtral sur LPU custom.",
        short_description_en: "Ultra-fast Llama / Mixtral inference on custom LPU.",
        default_max_tokens: None,
    },
    ProviderSpec {
        id: "google",
        display_name: "Google Gemini",
        category: "llm",
        // Couche OpenAI-compat officielle de Google
        base_url: "https://generativelanguage.googleapis.com/v1beta/openai",
        models_endpoint: "/models",
        signup_url: "https://aistudio.google.com/app/apikey",
        free_tier_label: "1M tokens/min",
        short_description: "Gemini 3.5 Flash — multimodal, tools et raisonnement.",
        short_description_en: "Gemini 3.5 Flash — multimodal, tools and reasoning.",
        default_max_tokens: None,
    },
    ProviderSpec {
        id: "mistral",
        display_name: "Mistral",
        category: "llm",
        base_url: "https://api.mistral.ai/v1",
        models_endpoint: "/models",
        signup_url: "https://console.mistral.ai/api-keys",
        free_tier_label: "1B tokens/month",
        short_description: "Modèles open-weight français — Experiment plan gratuit.",
        short_description_en: "French open-weight models — free Experiment plan.",
        default_max_tokens: Some(64_000),
    },
    ProviderSpec {
        id: "cerebras",
        display_name: "Cerebras",
        category: "llm",
        base_url: "https://api.cerebras.ai/v1",
        models_endpoint: "/models",
        signup_url: "https://cloud.cerebras.ai/",
        free_tier_label: "1M tokens/day",
        short_description: "Llama 3.3 à 450 tok/s sur wafer-scale chip.",
        short_description_en: "Llama 3.3 at 450 tok/s on wafer-scale chip.",
        default_max_tokens: None,
    },
    ProviderSpec {
        id: "openrouter",
        display_name: "OpenRouter",
        category: "llm",
        base_url: "https://openrouter.ai/api/v1",
        models_endpoint: "/models",
        signup_url: "https://openrouter.ai/settings/keys",
        free_tier_label: "30+ free models",
        short_description: "Accès à 300+ modèles via une seule clé API.",
        short_description_en: "Access 300+ models with a single API key.",
        default_max_tokens: Some(64_000),
    },
    ProviderSpec {
        id: "openai",
        display_name: "OpenAI",
        category: "llm",
        base_url: "https://api.openai.com/v1",
        models_endpoint: "/models",
        signup_url: "https://platform.openai.com/api-keys",
        free_tier_label: "$5 signup credits",
        short_description: "GPT-5.5, o-series reasoning models.",
        short_description_en: "GPT-5.5, o-series reasoning models.",
        default_max_tokens: Some(64_000),
    },
    ProviderSpec {
        id: "deepseek",
        display_name: "DeepSeek",
        category: "llm",
        base_url: "https://api.deepseek.com/v1",
        models_endpoint: "/models",
        signup_url: "https://platform.deepseek.com/api_keys",
        free_tier_label: "Low-cost ($0.14/Mtok)",
        short_description: "DeepSeek V4-Flash / V4-Pro — rapport qualité/prix imbattable.",
        short_description_en: "DeepSeek V4-Flash / V4-Pro — unbeatable value for money.",
        default_max_tokens: Some(64_000),
    },
    ProviderSpec {
        id: "xai",
        display_name: "xAI",
        category: "llm",
        base_url: "https://api.x.ai/v1",
        models_endpoint: "",
        signup_url: "https://console.x.ai",
        free_tier_label: "Budget ($0.20/Mtok)",
        short_description: "Grok 4.x — contexte 2M, raisonnement avancé.",
        short_description_en: "Grok 4.x — 2M context, advanced reasoning.",
        default_max_tokens: Some(64_000),
    },
    ProviderSpec {
        id: "moonshot",
        display_name: "Moonshot Kimi",
        category: "llm",
        base_url: "https://api.moonshot.ai/v1",
        models_endpoint: "/models",
        signup_url: "https://platform.kimi.ai/console/api-keys",
        free_tier_label: "Low-cost ($0.60/Mtok)",
        short_description: "Kimi K2.7 Code — agentic coding, multimodal, contexte 256K.",
        short_description_en: "Kimi K2.7 Code — agentic coding, multimodal, 256K context.",
        default_max_tokens: Some(64_000),
    },
    ProviderSpec {
        id: "zai",
        display_name: "Z.ai GLM",
        category: "llm",
        base_url: "https://api.z.ai/api/paas/v4",
        models_endpoint: "",
        signup_url: "https://z.ai/manage-apikey/apikey-list",
        free_tier_label: "GLM-4.5-Flash gratuit",
        short_description: "GLM-5.2 — contexte 1M, coding long-horizon et raisonnement.",
        short_description_en: "GLM-5.2 — 1M context, long-horizon coding and reasoning.",
        default_max_tokens: Some(64_000),
    },
];
