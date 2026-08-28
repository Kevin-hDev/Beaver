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
    /// Active l'ajout automatique d'un plafond quand l'appelant n'en fournit pas.
    /// Désactivé chez Cerebras : un plafond élevé réserve inutilement son TPM.
    pub auto_max_tokens: bool,
    /// Dernier repli si le registre Beaver, l'endpoint `/models` et LiteLLM
    /// ne publient aucune limite fiable. Il sera borné au contexte restant.
    pub fallback_max_tokens: Option<u32>,
}

/// Retourne la spec d'un provider LLM par son id.
pub fn find(provider_id: &str) -> Option<&'static ProviderSpec> {
    LLM_PROVIDERS.iter().find(|p| p.id == provider_id)
}

pub const LLM_PROVIDERS: &[ProviderSpec] = &[
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
        auto_max_tokens: true,
        fallback_max_tokens: None,
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
        // Mistral ne publie pas de plafond de sortie distinct du contexte du
        // modèle (vérifié le 2026-07-30) : dernier repli, borné au contexte restant.
        auto_max_tokens: true,
        fallback_max_tokens: Some(64_000),
    },
    ProviderSpec {
        id: "cerebras",
        display_name: "Cerebras",
        category: "llm",
        base_url: "https://api.cerebras.ai/v1",
        models_endpoint: "/models",
        signup_url: "https://cloud.cerebras.ai/",
        auto_max_tokens: false,
        fallback_max_tokens: None,
    },
    ProviderSpec {
        id: "openrouter",
        display_name: "OpenRouter",
        category: "llm",
        base_url: "https://openrouter.ai/api/v1",
        models_endpoint: "/models",
        signup_url: "https://openrouter.ai/settings/keys",
        // Dernier repli pour un modèle absent du registre Beaver et de LiteLLM.
        // OpenRouter revend des centaines de modèles aux plafonds très différents.
        auto_max_tokens: true,
        fallback_max_tokens: Some(64_000),
    },
    ProviderSpec {
        id: "openai",
        display_name: "OpenAI",
        category: "llm",
        base_url: "https://api.openai.com/v1",
        models_endpoint: "/models",
        signup_url: "https://platform.openai.com/api-keys",
        // GPT-5.6 Sol, Terra et Luna : 128k de sortie chacun (vérifié le 2026-07-30).
        auto_max_tokens: true,
        fallback_max_tokens: Some(128_000),
    },
    ProviderSpec {
        id: "deepseek",
        display_name: "DeepSeek",
        category: "llm",
        base_url: "https://api.deepseek.com/v1",
        models_endpoint: "/models",
        signup_url: "https://platform.deepseek.com/api_keys",
        // V4-Flash et V4-Pro : 384k de sortie chacun (vérifié le 2026-07-30).
        auto_max_tokens: true,
        fallback_max_tokens: Some(384_000),
    },
    ProviderSpec {
        id: "xai",
        display_name: "xAI",
        category: "llm",
        base_url: "https://api.x.ai/v1",
        // Endpoint officiel : le registre local enrichit cette liste dynamique.
        models_endpoint: "/models",
        signup_url: "https://console.x.ai",
        // xAI ne publie pas de plafond de sortie distinct du contexte
        // (vérifié le 2026-07-30) : dernier repli, borné au contexte restant.
        auto_max_tokens: true,
        fallback_max_tokens: Some(64_000),
    },
    ProviderSpec {
        id: "moonshot",
        display_name: "Moonshot Kimi",
        category: "llm",
        base_url: "https://api.moonshot.ai/v1",
        models_endpoint: "/models",
        signup_url: "https://platform.kimi.ai/console/api-keys",
        // Défaut documenté de Kimi K3, extensible à 1M (vérifié le 2026-07-30).
        // Les générations K2 antérieures ne publient pas leur plafond.
        auto_max_tokens: true,
        fallback_max_tokens: Some(131_072),
    },
    ProviderSpec {
        id: "zai",
        display_name: "Z.ai GLM",
        category: "llm",
        base_url: "https://api.z.ai/api/paas/v4",
        models_endpoint: "",
        signup_url: "https://z.ai/manage-apikey/apikey-list",
        // Repli réservé aux modèles futurs absents du registre local.
        auto_max_tokens: true,
        fallback_max_tokens: Some(96_000),
    },
];
