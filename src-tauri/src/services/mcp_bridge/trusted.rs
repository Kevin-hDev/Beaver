use std::collections::BTreeMap;
use std::sync::LazyLock;

static CANONICAL_ENDPOINTS: LazyLock<Result<BTreeMap<String, String>, serde_json::Error>> =
    LazyLock::new(|| {
        serde_json::from_str(include_str!("../../../../src/config/mcp-endpoints.json"))
    });

pub fn canonical_endpoint(connector_id: &str) -> Option<&'static str> {
    CANONICAL_ENDPOINTS
        .as_ref()
        .ok()?
        .get(connector_id)
        .map(String::as_str)
}

pub fn is_trusted_endpoint_for_connector(connector_id: &str, url: &str) -> bool {
    url.len() <= 2048 && canonical_endpoint(connector_id) == Some(url)
}

#[cfg(test)]
mod tests {
    use super::{canonical_endpoint, is_trusted_endpoint_for_connector};

    #[test]
    fn accepts_all_canonical_endpoints() {
        let endpoints: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(include_str!("../../../../src/config/mcp-endpoints.json"))
                .unwrap();
        assert_eq!(endpoints.len(), 13);
        for (id, value) in endpoints {
            let endpoint = value.as_str().unwrap();
            assert_eq!(canonical_endpoint(&id), Some(endpoint));
            assert!(is_trusted_endpoint_for_connector(&id, endpoint));
        }
    }

    #[test]
    fn rejects_noncanonical_or_ambiguous_urls() {
        for url in [
            "https://mcp.lucid.app",
            "https://mcp.lucid.app/",
            "https://mcp.lucid.app/mcp/extra",
            "https://mcp.lucid.app/mcp?query=1",
            "https://mcp.lucid.app/mcp#fragment",
            "https://user@mcp.lucid.app/mcp",
            "https://mcp.lucid.app:443/mcp",
            "http://mcp.lucid.app/mcp",
        ] {
            assert!(!is_trusted_endpoint_for_connector("lucid", url), "{url}");
        }
        assert!(!is_trusted_endpoint_for_connector(
            "notion",
            canonical_endpoint("lucid").unwrap()
        ));
        assert!(!is_trusted_endpoint_for_connector(
            "github",
            "https://api.githubcopilot.com/mcp"
        ));
    }
}
