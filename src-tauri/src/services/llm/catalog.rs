//! Catalogue statique des providers LLM free-tier supportés.
//!
//! Chaque `ProviderSpec` porte l'endpoint, l'URL d'inscription et les plafonds :
//! de quoi construire un `OpenAiCompatProvider` à partir d'un `provider_id` +
//! clé, et identifier le provider côté interface.
//!
//! Les textes affichés (description, palier gratuit) sont dans `src/i18n/*.json`
//! sous `apiKeys.providers.<id>` — un texte traduisible ne vit pas dans du code
//! compilé, où les sept langues seraient hors de portée des traducteurs.

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ProviderSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub category: &'static str, // "llm"
    pub base_url: &'static str,
    pub models_endpoint: &'static str, // chemin relatif à base_url
    pub signup_url: &'static str,
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
        default_max_tokens: None,
    },
    ProviderSpec {
        id: "google",
        display_name: "Google Gemini",
        category: "llm",
        // Vérifié le 2026-07-30 : le palier gratuit affiché (i18n) n'est plus
        // publiable — Google renvoie vers le tableau de bord AI Studio, visible
        // seulement après connexion. Le chiffre repose sur des sources tierces.
        // Couche OpenAI-compat officielle de Google
        base_url: "https://generativelanguage.googleapis.com/v1beta/openai",
        models_endpoint: "/models",
        signup_url: "https://aistudio.google.com/app/apikey",
        default_max_tokens: None,
    },
    ProviderSpec {
        id: "mistral",
        display_name: "Mistral",
        category: "llm",
        // Vérifié le 2026-07-30 : le palier gratuit affiché (i18n) a disparu de
        // la page tarifaire publique, il n'est plus visible que dans la console
        // après connexion. Le chiffre repose sur des sources tierces.
        base_url: "https://api.mistral.ai/v1",
        models_endpoint: "/models",
        signup_url: "https://console.mistral.ai/api-keys",
        default_max_tokens: Some(64_000),
    },
    ProviderSpec {
        id: "cerebras",
        display_name: "Cerebras",
        category: "llm",
        base_url: "https://api.cerebras.ai/v1",
        models_endpoint: "/models",
        signup_url: "https://cloud.cerebras.ai/",
        default_max_tokens: None,
    },
    ProviderSpec {
        id: "openrouter",
        display_name: "OpenRouter",
        category: "llm",
        base_url: "https://openrouter.ai/api/v1",
        models_endpoint: "/models",
        signup_url: "https://openrouter.ai/settings/keys",
        default_max_tokens: Some(64_000),
    },
    ProviderSpec {
        id: "openai",
        display_name: "OpenAI",
        category: "llm",
        base_url: "https://api.openai.com/v1",
        models_endpoint: "/models",
        signup_url: "https://platform.openai.com/api-keys",
        default_max_tokens: Some(64_000),
    },
    ProviderSpec {
        id: "deepseek",
        display_name: "DeepSeek",
        category: "llm",
        base_url: "https://api.deepseek.com/v1",
        models_endpoint: "/models",
        signup_url: "https://platform.deepseek.com/api_keys",
        default_max_tokens: Some(64_000),
    },
    ProviderSpec {
        id: "xai",
        display_name: "xAI",
        category: "llm",
        base_url: "https://api.x.ai/v1",
        // xAI expose désormais `GET /v1/models` (vérifié le 2026-07-30), mais le
        // remplir changerait aussi la façon de tester une clé (cf.
        // api_keys_http::test_key_raw) sans qu'on puisse l'essayer sans clé xAI.
        // La liste vient donc encore de XAI_MODELS.
        models_endpoint: "",
        signup_url: "https://console.x.ai",
        default_max_tokens: Some(64_000),
    },
    ProviderSpec {
        id: "moonshot",
        display_name: "Moonshot Kimi",
        category: "llm",
        base_url: "https://api.moonshot.ai/v1",
        models_endpoint: "/models",
        signup_url: "https://platform.kimi.ai/console/api-keys",
        default_max_tokens: Some(64_000),
    },
    ProviderSpec {
        id: "zai",
        display_name: "Z.ai GLM",
        category: "llm",
        base_url: "https://api.z.ai/api/paas/v4",
        models_endpoint: "",
        signup_url: "https://z.ai/manage-apikey/apikey-list",
        default_max_tokens: Some(64_000),
    },
];
