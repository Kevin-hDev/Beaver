use super::tool_result_contract::ToolErrorCategory;
use super::types_tools::ToolResult;

#[path = "tool_web_fetch_error.rs"]
mod fetch_error;

pub(super) fn search(message: String) -> ToolResult {
    if message == crate::services::searxng::error_codes::SHUTTING_DOWN {
        return error(
            message,
            "web_search_cancelled",
            ToolErrorCategory::Cancelled,
            false,
        );
    }
    if searxng_runtime_error(&message) {
        return error(
            message,
            "web_search_runtime_unavailable",
            ToolErrorCategory::Unavailable,
            true,
        );
    }
    if message == crate::services::searxng::error_codes::SEARCH_RATE_LIMITED {
        return error(
            message,
            "web_search_rate_limited",
            ToolErrorCategory::External,
            true,
        );
    }
    if message == crate::services::searxng::error_codes::SEARCH_INVALID_RESPONSE {
        return error(
            message,
            "web_search_invalid_response",
            ToolErrorCategory::External,
            true,
        );
    }
    let lower = message.to_lowercase();
    if contains_any(&lower, &["requête vide", "requête trop longue"]) {
        return error(
            message,
            "invalid_web_search_query",
            ToolErrorCategory::Validation,
            false,
        );
    }
    if lower.contains("aucun provider configuré") {
        return error(
            message,
            "web_search_not_configured",
            ToolErrorCategory::Unavailable,
            false,
        )
        .with_error_hint("Configurer un fournisseur de recherche ou rendre SearXNG disponible.");
    }
    if http_status(&message) == Some(429) || lower.contains("limite de requêtes") {
        return error(
            message,
            "web_search_rate_limited",
            ToolErrorCategory::External,
            true,
        )
        .with_error_hint("Réessayer plus tard ou utiliser une autre source.");
    }
    if contains_any(&lower, &["timeout", "délai dépassé"]) {
        return error(
            message,
            "web_search_timeout",
            ToolErrorCategory::Timeout,
            true,
        );
    }
    if contains_any(&lower, &["authentification", "clé ", "api key"]) {
        return error(
            message,
            "web_search_auth_failed",
            ToolErrorCategory::Permission,
            false,
        );
    }
    if lower.contains("réponse invalide") {
        return error(
            message,
            "web_search_invalid_response",
            ToolErrorCategory::External,
            true,
        );
    }
    error(
        message,
        "web_search_unavailable",
        ToolErrorCategory::External,
        true,
    )
}

fn searxng_runtime_error(message: &str) -> bool {
    use crate::services::searxng::error_codes;

    [
        error_codes::APP_UNAVAILABLE,
        error_codes::BUNDLE_INVALID,
        error_codes::CONFIG_UNAVAILABLE,
        error_codes::LOG_UNAVAILABLE,
        error_codes::OPERATION_INTERRUPTED,
        error_codes::PROCESS_STATE_UNAVAILABLE,
        error_codes::RUNTIME_UNAVAILABLE,
        error_codes::SETTINGS_UNAVAILABLE,
        error_codes::SOURCE_UNAVAILABLE,
        error_codes::START_FAILED,
    ]
    .contains(&message)
}

pub(super) fn fetch(message: String) -> ToolResult {
    fetch_error::classify(message)
}

fn http_status(message: &str) -> Option<u16> {
    let mut parts = message.split_whitespace();
    while let Some(part) = parts.next() {
        if part.eq_ignore_ascii_case("HTTP") {
            return parts
                .next()
                .and_then(|value| value.trim_matches(|c: char| !c.is_ascii_digit()).parse().ok());
        }
    }
    None
}

fn error(
    message: String,
    code: &'static str,
    category: ToolErrorCategory,
    retryable: bool,
) -> ToolResult {
    ToolResult::error(message, code, category, retryable)
}

fn contains_any(message: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| message.contains(pattern))
}

#[cfg(test)]
#[path = "tool_web_error_tests.rs"]
mod tests;
