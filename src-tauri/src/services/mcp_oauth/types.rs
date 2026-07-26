use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

#[derive(Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    #[serde(default = "default_bearer")]
    pub token_type: String,
}

fn default_bearer() -> String {
    "Bearer".to_string()
}

impl Drop for TokenResponse {
    fn drop(&mut self) {
        self.access_token.zeroize();
        if let Some(ref mut rt) = self.refresh_token {
            rt.zeroize();
        }
        self.token_type.zeroize();
    }
}

#[derive(Serialize, Deserialize)]
pub struct OAuthTokensSerde {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
    pub token_type: String,
    pub token_endpoint: String,
    pub client_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
}

impl Drop for OAuthTokensSerde {
    fn drop(&mut self) {
        self.access_token.zeroize();
        if let Some(ref mut rt) = self.refresh_token {
            rt.zeroize();
        }
        self.token_type.zeroize();
        self.token_endpoint.zeroize();
        self.client_id.zeroize();
        if let Some(ref mut cs) = self.client_secret {
            cs.zeroize();
        }
    }
}

pub struct OAuthTokens {
    pub access_token: Zeroizing<String>,
    pub refresh_token: Option<Zeroizing<String>>,
    pub expires_at: Option<i64>,
    pub token_type: String,
    pub token_endpoint: String,
    pub client_id: String,
    pub client_secret: Option<Zeroizing<String>>,
}

impl OAuthTokens {
    pub fn to_json(&self) -> Result<Zeroizing<String>, String> {
        let raw = OAuthTokensSerde {
            access_token: self.access_token.as_str().to_string(),
            refresh_token: self.refresh_token.as_ref().map(|s| s.as_str().to_string()),
            expires_at: self.expires_at,
            token_type: self.token_type.clone(),
            token_endpoint: self.token_endpoint.clone(),
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.as_ref().map(|s| s.as_str().to_string()),
        };
        let json = serde_json::to_string(&raw).map_err(|_| "erreur interne".to_string())?;
        Ok(Zeroizing::new(json))
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        let mut raw: OAuthTokensSerde = serde_json::from_str(json)
            .map_err(|_| "données d'authentification invalides".to_string())?;
        let tokens = Self {
            access_token: Zeroizing::new(std::mem::take(&mut raw.access_token)),
            refresh_token: raw.refresh_token.take().map(Zeroizing::new),
            expires_at: raw.expires_at,
            token_type: std::mem::take(&mut raw.token_type),
            token_endpoint: std::mem::take(&mut raw.token_endpoint),
            client_id: std::mem::take(&mut raw.client_id),
            client_secret: raw.client_secret.take().map(Zeroizing::new),
        };
        Ok(tokens)
    }

    pub fn from_response(
        resp: &mut TokenResponse,
        token_endpoint: &str,
        client_id: &str,
        client_secret: Option<&str>,
    ) -> Self {
        Self {
            access_token: Zeroizing::new(std::mem::take(&mut resp.access_token)),
            refresh_token: resp.refresh_token.take().map(Zeroizing::new),
            expires_at: resp.expires_in.map(|d| chrono::Utc::now().timestamp() + d),
            token_type: std::mem::take(&mut resp.token_type),
            token_endpoint: token_endpoint.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.map(|s| Zeroizing::new(s.to_string())),
        }
    }
}

impl Drop for OAuthTokens {
    fn drop(&mut self) {
        self.token_endpoint.zeroize();
        self.client_id.zeroize();
        self.token_type.zeroize();
    }
}

#[derive(Deserialize)]
pub struct AuthServerMetadata {
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub registration_endpoint: Option<String>,
    #[allow(dead_code)]
    pub code_challenge_methods_supported: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct ProtectedResourceMetadata {
    pub authorization_servers: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct DcrResponse {
    pub client_id: String,
    #[allow(dead_code)]
    pub client_secret: Option<String>,
}

impl Drop for DcrResponse {
    fn drop(&mut self) {
        self.client_id.zeroize();
        if let Some(ref mut s) = self.client_secret {
            s.zeroize();
        }
    }
}

pub struct CallbackResult {
    pub code: Zeroizing<String>,
    pub state: Zeroizing<String>,
}
