use super::tool_result_contract::ToolErrorCategory;
use super::types_tools::ToolResult;

pub(super) fn search(message: String) -> ToolResult {
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

pub(super) fn fetch(message: String) -> ToolResult {
    let lower = message.to_lowercase();
    if let Some(status) = http_status(&message) {
        return http_fetch_error(message, status);
    }
    if lower.contains("timeout") {
        return error(
            message,
            "web_fetch_timeout",
            ToolErrorCategory::Timeout,
            true,
        );
    }
    if contains_any(
        &lower,
        &[
            "url invalide",
            "url trop longue",
            "schéma non autorisé",
            "hôte manquant",
            "type de contenu non supporté",
        ],
    ) {
        return error(
            message,
            "invalid_web_fetch_request",
            ToolErrorCategory::Validation,
            false,
        );
    }
    if contains_any(
        &lower,
        &[
            "identifiants url interdits",
            "cloud metadata bloqué",
            "adresse privée bloquée",
            "port non autorisé",
        ],
    ) {
        return error(
            message,
            "web_fetch_url_blocked",
            ToolErrorCategory::Permission,
            false,
        );
    }
    if lower.contains("trop de redirections") || lower.contains("redirection invalide") {
        return error(
            message,
            "web_fetch_redirect_failed",
            ToolErrorCategory::External,
            false,
        );
    }
    if lower.contains("réponse trop volumineuse") {
        return error(
            message,
            "web_fetch_response_too_large",
            ToolErrorCategory::External,
            false,
        );
    }
    error(
        message,
        "web_fetch_transport_failed",
        ToolErrorCategory::External,
        true,
    )
}

fn http_fetch_error(message: String, status: u16) -> ToolResult {
    match status {
        401 | 403 => error(
            message,
            "web_fetch_access_denied",
            ToolErrorCategory::Permission,
            false,
        ),
        404 | 410 => error(
            message,
            "web_fetch_not_found",
            ToolErrorCategory::NotFound,
            false,
        ),
        408 => error(
            message,
            "web_fetch_timeout",
            ToolErrorCategory::Timeout,
            true,
        ),
        429 => error(
            message,
            "web_fetch_rate_limited",
            ToolErrorCategory::External,
            true,
        ),
        500..=599 => error(
            message,
            "web_fetch_server_error",
            ToolErrorCategory::External,
            true,
        ),
        _ => error(
            message,
            "web_fetch_http_error",
            ToolErrorCategory::External,
            false,
        ),
    }
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
