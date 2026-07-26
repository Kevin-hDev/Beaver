use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderId {
    OpenAi,
    Moonshot,
    Xai,
}

impl ProviderId {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "openai" => Ok(Self::OpenAi),
            "moonshot" => Ok(Self::Moonshot),
            "xai" => Ok(Self::Xai),
            _ => Err("Fournisseur invalide".to_string()),
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::OpenAi => "OpenAI",
            Self::Moonshot => "Moonshot AI",
            Self::Xai => "xAI",
        }
    }

    pub const fn usage_connection_id(self) -> &'static str {
        match self {
            Self::OpenAi => "codex-oauth",
            Self::Moonshot => "moonshot-oauth",
            Self::Xai => "xai-oauth",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OAuthProviderStatus {
    pub id: ProviderId,
    pub display_name: &'static str,
    pub connected: bool,
    pub account: Option<String>,
    pub experimental: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct OAuthLoginProgress {
    pub provider_id: ProviderId,
    pub stage: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_code: Option<String>,
}
