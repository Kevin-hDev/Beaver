pub fn is_service_tier_rejection(body: &str) -> bool {
    let Ok(document) = serde_json::from_str::<serde_json::Value>(body) else {
        return false;
    };
    service_tier_error_fields(
        document.pointer("/error/param"),
        document.pointer("/error/code"),
    )
}

pub fn is_service_tier_response_error(event: &serde_json::Value) -> bool {
    service_tier_error_fields(
        event.pointer("/response/error/param"),
        event.pointer("/response/error/code"),
    )
}

fn service_tier_error_fields(
    param: Option<&serde_json::Value>,
    code: Option<&serde_json::Value>,
) -> bool {
    if param.and_then(serde_json::Value::as_str) == Some("service_tier") {
        return true;
    }
    // Hypothèse défensive fermée, à retirer si la campagne réelle ne l'observe pas.
    code.and_then(serde_json::Value::as_str) == Some("unsupported_service_tier")
}
