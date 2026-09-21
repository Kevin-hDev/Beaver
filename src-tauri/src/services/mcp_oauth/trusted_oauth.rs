use super::types::AuthServerMetadata;

pub fn validate_metadata_endpoints(
    connector_id: &str,
    meta: &AuthServerMetadata,
) -> Result<(), String> {
    validate_issuer(connector_id, &meta.issuer)?;
    if let Some(methods) = &meta.code_challenge_methods_supported {
        if methods.is_empty()
            || methods.len() > 16
            || methods.iter().any(|method| method.len() > 32)
            || !methods.iter().any(|method| method == "S256")
        {
            return Err("métadonnées OAuth invalides".to_string());
        }
    }
    validate_endpoint(connector_id, &meta.authorization_endpoint)?;
    validate_endpoint(connector_id, &meta.token_endpoint)?;
    if let Some(registration_endpoint) = &meta.registration_endpoint {
        validate_endpoint(connector_id, registration_endpoint)?;
    }
    Ok(())
}

pub fn validate_issuer(connector_id: &str, issuer: &str) -> Result<(), String> {
    validate_endpoint(connector_id, issuer)?;
    let parsed = reqwest::Url::parse(issuer).map_err(|_| "émetteur OAuth invalide")?;
    if parsed.query().is_some() || parsed.fragment().is_some() {
        return Err("émetteur OAuth invalide".to_string());
    }
    let required = match connector_id {
        "github" => Some("https://github.com/login/oauth"),
        "gmail" | "google-drive" | "google-calendar" => Some("https://accounts.google.com"),
        _ => None,
    };
    if required.is_some_and(|expected| issuer != expected) {
        return Err("émetteur OAuth invalide".to_string());
    }
    Ok(())
}

pub fn validate_endpoint(connector_id: &str, url: &str) -> Result<(), String> {
    if url.len() > 2048 {
        return Err("endpoint OAuth non autorisé".to_string());
    }
    let parsed = reqwest::Url::parse(url).map_err(|_| "endpoint OAuth invalide".to_string())?;
    if parsed.scheme() != "https"
        || parsed.port_or_known_default() != Some(443)
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.fragment().is_some()
    {
        return Err("endpoint OAuth non autorisé".to_string());
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| "endpoint OAuth invalide".to_string())?;
    if trusted_hosts(connector_id).contains(&host) {
        return Ok(());
    }
    Err("endpoint OAuth non autorisé".to_string())
}

fn trusted_hosts(connector_id: &str) -> &'static [&'static str] {
    // These exact hosts follow each provider's published OAuth metadata; never trust whole domains.
    match connector_id {
        "gmail" => &[
            "gmailmcp.googleapis.com",
            "accounts.google.com",
            "oauth2.googleapis.com",
        ],
        "google-drive" => &[
            "drivemcp.googleapis.com",
            "accounts.google.com",
            "oauth2.googleapis.com",
        ],
        "google-calendar" => &[
            "calendarmcp.googleapis.com",
            "accounts.google.com",
            "oauth2.googleapis.com",
        ],
        "canva" => &["mcp.canva.com", "www.canva.com", "canva.com"],
        "figma" => &[
            "mcp.figma.com",
            "www.figma.com",
            "figma.com",
            "api.figma.com",
        ],
        "notion" => &["mcp.notion.com", "api.notion.com", "notion.com"],
        "slack" => &["mcp.slack.com", "slack.com"],
        "linear" => &["mcp.linear.app", "linear.app"],
        "lucid" => &["mcp.lucid.app", "lucid.app", "lucid.co"],
        "sentry" => &["mcp.sentry.dev", "sentry.io", "sentry.dev"],
        "vercel" => &["mcp.vercel.com", "vercel.com", "api.vercel.com"],
        "apify" => &[
            "mcp.apify.com",
            "apify.com",
            "console.apify.com",
            "console-backend.apify.com",
        ],
        "github" => &["github.com"],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_endpoint, validate_issuer};

    #[test]
    fn accepts_vercel_figma_and_apify_oauth_metadata() {
        use crate::services::mcp_oauth::types::AuthServerMetadata;

        let providers = [
            (
                "vercel",
                "https://vercel.com",
                "https://vercel.com/oauth/authorize",
                "https://api.vercel.com/login/oauth/token",
                "https://api.vercel.com/login/oauth/register",
                false,
            ),
            (
                "figma",
                "https://api.figma.com",
                "https://www.figma.com/oauth/mcp",
                "https://api.figma.com/v1/oauth/token",
                "https://api.figma.com/v1/oauth/mcp/register",
                true,
            ),
            (
                "apify",
                "https://console-backend.apify.com",
                "https://console.apify.com/authorize/oauth",
                "https://console-backend.apify.com/oauth/apps/token",
                "https://console-backend.apify.com/oauth/apps",
                true,
            ),
        ];
        for (
            id,
            issuer,
            authorization_endpoint,
            token_endpoint,
            registration_endpoint,
            emits_iss,
        ) in providers
        {
            let meta = AuthServerMetadata {
                issuer: issuer.to_string(),
                authorization_response_iss_parameter_supported: emits_iss,
                authorization_endpoint: authorization_endpoint.to_string(),
                token_endpoint: token_endpoint.to_string(),
                registration_endpoint: Some(registration_endpoint.to_string()),
                code_challenge_methods_supported: Some(vec!["S256".to_string()]),
            };
            assert!(
                super::validate_metadata_endpoints(id, &meta).is_ok(),
                "{id}"
            );
        }
    }

    #[test]
    fn metadata_method_list_is_bounded_and_requires_pkce_s256() {
        let mut meta = crate::services::mcp_oauth::types::AuthServerMetadata {
            issuer: "https://mcp.notion.com".to_string(),
            authorization_response_iss_parameter_supported: false,
            authorization_endpoint: "https://mcp.notion.com/authorize".to_string(),
            token_endpoint: "https://mcp.notion.com/token".to_string(),
            registration_endpoint: None,
            code_challenge_methods_supported: Some(vec!["plain".to_string()]),
        };
        assert!(super::validate_metadata_endpoints("notion", &meta).is_err());
        meta.code_challenge_methods_supported = Some(vec!["S256".to_string(); 17]);
        assert!(super::validate_metadata_endpoints("notion", &meta).is_err());
    }

    #[test]
    fn static_issuer_identity_cannot_be_replaced_by_another_path() {
        assert!(validate_issuer("github", "https://github.com/login/oauth").is_ok());
        assert!(validate_issuer("github", "https://github.com/other").is_err());
        assert!(validate_issuer("gmail", "https://accounts.google.com").is_ok());
        assert!(validate_issuer("gmail", "https://accounts.google.com/tenant").is_err());
    }

    #[test]
    fn accepts_expected_connector_oauth_hosts() {
        assert!(validate_endpoint("github", "https://github.com/login/oauth/access_token").is_ok());
        assert!(validate_endpoint("gmail", "https://accounts.google.com/o/oauth2/v2/auth").is_ok());
        assert!(validate_endpoint("sentry", "https://sentry.io/oauth/authorize").is_ok());
    }

    #[test]
    fn rejects_connector_host_mismatch() {
        assert!(
            validate_endpoint("notion", "https://github.com/login/oauth/access_token").is_err()
        );
        assert!(validate_endpoint("sentry", "https://mcp.notion.com/oauth").is_err());
    }

    #[test]
    fn rejects_non_https_and_userinfo() {
        assert!(validate_endpoint("github", "http://github.com/login/oauth/access_token").is_err());
        assert!(validate_endpoint(
            "github",
            "https://token@github.com/login/oauth/access_token"
        )
        .is_err());
    }

    #[test]
    fn refuses_unlisted_subdomains_ports_and_fragments() {
        assert!(validate_endpoint("github", "https://attacker.github.com/oauth").is_err());
        assert!(validate_endpoint("github", "https://github.com:8443/oauth").is_err());
        assert!(validate_endpoint("github", "https://github.com/oauth#x").is_err());
    }
}
