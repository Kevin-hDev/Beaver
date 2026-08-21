//! Module Search / Scraping multi-provider.
//!
//! Le tool `web_search` de l'agent utilise `run_search` qui route vers
//! le premier provider configuré dans l'ordre de préférence :
//! Brave → Exa → Firecrawl → SearXNG (fallback local sans clé).

pub mod brave;
pub mod catalog;
pub mod common;
pub mod exa;
pub mod firecrawl;

use crate::services::agent_local::types_tools::SearchResult;
use crate::services::api_keys;
use std::future::Future;

#[derive(Debug)]
pub(crate) struct SearchFailure {
    message: String,
    machine_code: Option<&'static str>,
}

impl SearchFailure {
    pub(crate) fn plain(message: String) -> Self {
        Self {
            message,
            machine_code: None,
        }
    }

    pub(crate) fn searxng(code: &'static str) -> Self {
        Self {
            // A fixed code is safe at the tool boundary; the UI translates it.
            message: code.to_string(),
            machine_code: Some(code),
        }
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn machine_code(&self) -> Option<&'static str> {
        self.machine_code
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchProvider {
    Brave,
    Exa,
    Firecrawl,
}

impl SearchProvider {
    fn id(self) -> &'static str {
        match self {
            Self::Brave => "brave",
            Self::Exa => "exa",
            Self::Firecrawl => "firecrawl",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Brave => "Brave",
            Self::Exa => "Exa",
            Self::Firecrawl => "Firecrawl",
        }
    }
}

const PROVIDER_ORDER: [SearchProvider; 3] = [
    SearchProvider::Brave,
    SearchProvider::Exa,
    SearchProvider::Firecrawl,
];

/// Orchestrateur de recherche web — essaie chaque provider dans l'ordre.
pub(crate) async fn run_search(query: &str) -> Result<Vec<SearchResult>, SearchFailure> {
    let query = common::validate_query(query).map_err(SearchFailure::plain)?;
    let (configured, provider_succeeded, failures, provider_result) = try_configured_providers(
        &query,
        |provider| api_keys::has_key(provider.id()),
        |provider, query| async move { search_with_provider(provider, &query).await },
    )
    .await;
    if let Some(results) = provider_result {
        return Ok(results);
    }

    finish_search(
        configured,
        provider_succeeded,
        failures,
        crate::services::searxng::search(&query).await,
    )
}

pub(crate) fn finish_search(
    configured: bool,
    provider_succeeded: bool,
    mut failures: Vec<String>,
    searxng_result: Result<Vec<SearchResult>, String>,
) -> Result<Vec<SearchResult>, SearchFailure> {
    match searxng_result {
        Ok(results) if !results.is_empty() => return Ok(results),
        Ok(_) => return Ok(Vec::new()),
        Err(error) => {
            if provider_succeeded {
                return Ok(Vec::new());
            }
            match crate::services::searxng::error_codes::known(&error) {
                // Les erreurs des providers configurés sont plus actionnables
                // que l'état du repli local : ce code ne doit jamais les masquer.
                Some(code) if failures.is_empty() => return Err(SearchFailure::searxng(code)),
                Some(code) => {
                    ::log::warn!(
                        "[search] repli SearXNG indisponible code={code} providers_en_echec={}",
                        failures.len()
                    );
                    failures.push(code.to_string());
                }
                None => failures.push(common::sanitize_error(&error)),
            }
        }
    }

    if configured {
        Err(SearchFailure::plain(format_failures(&failures)))
    } else {
        Err(SearchFailure::plain(format!(
            "Aucun provider configuré. Fallback SearXNG indisponible: {}",
            format_failures(&failures)
        )))
    }
}

async fn try_configured_providers<HasKey, SearchFn, SearchFut>(
    query: &str,
    has_key: HasKey,
    mut search_fn: SearchFn,
) -> (bool, bool, Vec<String>, Option<Vec<SearchResult>>)
where
    HasKey: Fn(SearchProvider) -> bool,
    SearchFn: FnMut(SearchProvider, String) -> SearchFut,
    SearchFut: Future<Output = Result<Vec<SearchResult>, String>>,
{
    let mut failures = Vec::new();
    let mut configured = false;
    let mut succeeded = false;

    for provider in PROVIDER_ORDER {
        if !has_key(provider) {
            continue;
        }
        configured = true;
        match search_fn(provider, query.to_string()).await {
            Ok(results) if !results.is_empty() => {
                return (configured, true, failures, Some(results))
            }
            Ok(_) => {
                succeeded = true;
                failures.push(format!("{}: résultat vide", provider.label()));
            }
            Err(e) => failures.push(common::sanitize_error(&e)),
        }
    }

    (configured, succeeded, failures, None)
}

async fn search_with_provider(
    provider: SearchProvider,
    query: &str,
) -> Result<Vec<SearchResult>, String> {
    match provider {
        SearchProvider::Brave => brave::search(query).await,
        SearchProvider::Exa => exa::search(query).await,
        SearchProvider::Firecrawl => firecrawl::search(query).await,
    }
}

fn format_failures(failures: &[String]) -> String {
    if failures.is_empty() {
        return "Recherche web indisponible".to_string();
    }
    format!("Recherche web indisponible: {}", failures.join("; "))
}

/// Test de connexion d'un provider search spécifique (utilisé par l'UI
/// quand l'utilisateur colle une clé).
pub async fn test_connection(provider_id: &str) -> Result<(), String> {
    match provider_id {
        "brave" => brave::test_connection().await,
        "exa" => exa::test_connection().await,
        "firecrawl" => firecrawl::test_connection().await,
        other => Err(format!("Test non implémenté pour {}", other)),
    }
}

#[cfg(test)]
mod tests;
