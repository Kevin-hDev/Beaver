use serde_json::Value;

/// Web search/fetch tools — always enabled (locked).
pub fn web_tool_definitions() -> Vec<Value> {
    use super::tool_definitions::tool_def;
    vec![
        tool_def(
            "web_search",
            "Search the web for current information, documentation, or solutions. \
             Provider is selected automatically from the configured search APIs (Brave, Exa, Firecrawl, in that order) based on which API key is set. \
             You cannot select a provider or filter by domain. \
             Returns a list of results (title, url, snippet). Result count depends on the provider. \
             Keep queries concise and specific. Use the current year for time-sensitive topics. \
             At least one search provider must be configured in settings.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Search query"}
                },
                "required": ["query"]
            }),
        ),
        tool_def(
            "web_fetch",
            "Fetch a URL and extract its main content as text/markdown. \
             For HTML pages: extracts the readable main content (article-style), with a fallback to full-page markdown (scripts/styles stripped). \
             For JSON/XML/plain text: returns the body verbatim. \
             Other content types are rejected. \
             Limits: response body max 5 MB; output max 50,000 chars (truncated). \
             No HTTP->HTTPS upgrade — both schemes are accepted. \
             Redirects followed up to 3 times, each re-validated against SSRF rules (private IPs blocked). \
             No caching. \
             Requires user confirmation unless session-allowed.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {"type": "string", "description": "URL to fetch"}
                },
                "required": ["url"]
            }),
        ),
    ]
}
