use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use zeroize::Zeroizing;

const MAX_JWT_BYTES: usize = 512 * 1024;
const MAX_ACCOUNT_HINT: usize = 128;
const MAX_EMAIL: usize = 320;

/// Données non vérifiées, réservées à l'affichage et au routage côté serveur.
/// Elles ne constituent jamais une preuve locale d'identité.
pub struct JwtDisplayClaims {
    pub account_hint: String,
    pub email: Option<String>,
}

pub fn extract_display_claims(jwt: &str) -> Result<JwtDisplayClaims, String> {
    if jwt.is_empty() || jwt.len() > MAX_JWT_BYTES {
        return Err("jeton d'affichage invalide".to_string());
    }
    let mut parts = jwt.split('.');
    let _header = parts.next();
    let payload = parts
        .next()
        .ok_or_else(|| "jeton d'affichage invalide".to_string())?;
    if parts.next().is_none() || parts.next().is_some() || payload.len() > MAX_JWT_BYTES {
        return Err("jeton d'affichage invalide".to_string());
    }
    let payload = Zeroizing::new(
        URL_SAFE_NO_PAD
            .decode(payload)
            .or_else(|_| {
                use base64::engine::general_purpose::URL_SAFE;
                URL_SAFE.decode(payload)
            })
            .map_err(|_| "jeton d'affichage invalide".to_string())?,
    );
    if payload.len() > MAX_JWT_BYTES {
        return Err("jeton d'affichage invalide".to_string());
    }
    let json: serde_json::Value =
        serde_json::from_slice(&payload).map_err(|_| "jeton d'affichage invalide".to_string())?;
    let account_hint = json["https://api.openai.com/auth"]["chatgpt_account_id"]
        .as_str()
        .filter(|value| valid_account_hint(value))
        .ok_or_else(|| "jeton d'affichage invalide".to_string())?
        .to_string();
    let email = json["https://api.openai.com/profile"]["email"]
        .as_str()
        .filter(|value| !value.is_empty() && value.len() <= MAX_EMAIL)
        .map(String::from);
    Ok(JwtDisplayClaims {
        account_hint,
        email,
    })
}

fn valid_account_hint(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ACCOUNT_HINT
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

#[cfg(test)]
#[path = "jwt_tests.rs"]
mod tests;
