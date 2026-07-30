//! Catalogue statique des providers Search / Scraping.
//!
//! Comme pour les providers LLM, les textes affichés vivent dans
//! `src/i18n/*.json` sous `apiKeys.providers.<id>` ; ici on ne garde que
//! l'identité et l'URL d'inscription.

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SearchProviderSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub category: &'static str, // "search" | "scraping"
    pub signup_url: &'static str,
}

pub const SEARCH_PROVIDERS: &[SearchProviderSpec] = &[
    SearchProviderSpec {
        id: "brave",
        display_name: "Brave Search",
        category: "search",
        signup_url: "https://api-dashboard.search.brave.com/app/keys",
    },
    SearchProviderSpec {
        id: "exa",
        display_name: "Exa",
        category: "search",
        signup_url: "https://dashboard.exa.ai/api-keys",
    },
    SearchProviderSpec {
        id: "firecrawl",
        display_name: "Firecrawl",
        category: "scraping",
        signup_url: "https://www.firecrawl.dev/app/api-keys",
    },
];
