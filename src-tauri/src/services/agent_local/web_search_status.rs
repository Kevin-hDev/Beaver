//! Builds the "Web search status" section injected into the system prompt so
//! the LLM knows which search providers are currently usable.

/// Build the web-search status section from raw provider booleans.
///
/// Extracted as a pure function so it can be unit-tested without touching the
/// process-global API-key state. The caller (`prepend_web_search_status` in
/// `chat_prompts.rs`) reads the booleans from `api_keys::has_key` and
/// `searxng::is_ready` then forwards them here.
pub fn format_web_search_status(brave: bool, exa: bool, firecrawl: bool, searxng: bool) -> String {
    let mut active: Vec<&str> = Vec::new();
    if brave {
        active.push("Brave");
    }
    if exa {
        active.push("Exa");
    }
    if firecrawl {
        active.push("Firecrawl");
    }
    if searxng {
        active.push("SearXNG (local fallback)");
    }

    if active.is_empty() {
        "\n\n## Web search status\n\
         No web search provider is configured. The web_search tool WILL FAIL. \
         Tell the user they can configure Brave, Exa, or Firecrawl in Settings → API keys, \
         or wait for the local SearXNG fallback to become available."
            .to_string()
    } else {
        format!(
            "\n\n## Web search status\n\
             Active providers: {}. Provider is selected automatically — you cannot pick one.",
            active.join(", ")
        )
    }
}
