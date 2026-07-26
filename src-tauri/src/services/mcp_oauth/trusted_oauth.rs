use super::types::AuthServerMetadata;

pub fn validate_metadata_endpoints(
    connector_id: &str,
    meta: &AuthServerMetadata,
) -> Result<(), String> {
    validate_endpoint(connector_id, &meta.authorization_endpoint)?;
    validate_endpoint(connector_id, &meta.token_endpoint)?;
    if let Some(registration_endpoint) = &meta.registration_endpoint {
        validate_endpoint(connector_id, registration_endpoint)?;
    }
    Ok(())
}

pub fn validate_endpoint(connector_id: &str, url: &str) -> Result<(), String> {
    let parsed = reqwest::Url::parse(url).map_err(|_| "endpoint OAuth invalide".to_string())?;
    if parsed.scheme() != "https" || !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("endpoint OAuth non autorisé".to_string());
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| "endpoint OAuth invalide".to_string())?;
    if trusted_hosts(connector_id)
        .iter()
        .any(|trusted| host_matches(host, trusted))
    {
        return Ok(());
    }
    Err("endpoint OAuth non autorisé".to_string())
}

fn host_matches(host: &str, trusted: &str) -> bool {
    host == trusted || host.ends_with(&format!(".{trusted}"))
}

fn trusted_hosts(connector_id: &str) -> &'static [&'static str] {
    match connector_id {
        "gmail" | "google-drive" | "google-calendar" => &["google.com", "googleapis.com"],
        "canva" => &["canva.com"],
        "figma" => &["figma.com"],
        "notion" => &["notion.com"],
        "slack" => &["slack.com"],
        "linear" => &["linear.app"],
        "lucid" => &["lucid.app", "lucid.co"],
        "sentry" => &["sentry.dev", "sentry.io"],
        "vercel" => &["vercel.com"],
        "apify" => &["apify.com"],
        "github" => &["github.com"],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::validate_endpoint;

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
}
