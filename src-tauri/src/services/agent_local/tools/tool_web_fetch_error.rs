use super::{contains_any, error, http_status};
use crate::services::agent_local::tool_result_contract::ToolErrorCategory;
use crate::services::agent_local::types_tools::ToolResult;

pub(super) fn classify(message: String) -> ToolResult {
    let lower = message.to_lowercase();
    if let Some(status) = http_status(&message) {
        return http_error(message, status);
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

fn http_error(message: String, status: u16) -> ToolResult {
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
