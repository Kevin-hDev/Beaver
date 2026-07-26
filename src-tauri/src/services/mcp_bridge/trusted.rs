const TRUSTED_ENDPOINTS: &[(&str, &str, &str)] = &[
    ("gmail", "gmailmcp.googleapis.com", "/mcp/v1"),
    ("google-drive", "drivemcp.googleapis.com", "/mcp/v1"),
    ("google-calendar", "calendarmcp.googleapis.com", "/mcp/v1"),
    ("canva", "mcp.canva.com", "/mcp"),
    ("figma", "mcp.figma.com", "/mcp"),
    ("notion", "mcp.notion.com", "/mcp"),
    ("slack", "mcp.slack.com", "/mcp"),
    ("linear", "mcp.linear.app", "/mcp"),
    ("lucid", "mcp.lucid.app", "/"),
    ("sentry", "mcp.sentry.dev", "/mcp"),
    ("vercel", "mcp.vercel.com", "/"),
    ("apify", "mcp.apify.com", "/"),
    ("github", "api.githubcopilot.com", "/mcp"),
];

pub fn is_trusted_endpoint_for_connector(connector_id: &str, url: &str) -> bool {
    let parsed = match reqwest::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return false,
    };
    if parsed.scheme() != "https" || parsed.query().is_some() || parsed.port().is_some() {
        return false;
    }
    let Some(host) = parsed.host_str() else {
        return false;
    };
    let path = normalize_path(parsed.path());
    TRUSTED_ENDPOINTS
        .iter()
        .any(|(id, h, p)| *id == connector_id && *h == host && *p == path)
}

fn normalize_path(path: &str) -> &str {
    if path != "/" && path.ends_with('/') {
        &path[..path.len() - 1]
    } else {
        path
    }
}

#[cfg(test)]
mod tests {
    use super::is_trusted_endpoint_for_connector;

    #[test]
    fn accepts_catalog_endpoints() {
        assert!(is_trusted_endpoint_for_connector(
            "notion",
            "https://mcp.notion.com/mcp"
        ));
        assert!(is_trusted_endpoint_for_connector(
            "lucid",
            "https://mcp.lucid.app"
        ));
        assert!(is_trusted_endpoint_for_connector(
            "sentry",
            "https://mcp.sentry.dev/mcp"
        ));
    }

    #[test]
    fn rejects_connector_endpoint_mismatch() {
        assert!(!is_trusted_endpoint_for_connector(
            "notion",
            "https://mcp.sentry.dev/mcp"
        ));
    }
}
