const MAX_FIXTURE_ID_BYTES: usize = 240;

/// Le rapport conserve le modèle exact ; son nom est un slug sans chemin et stable.
pub(crate) fn derive_fixture_id(
    provider: &str,
    model: &str,
    region: &str,
    date: chrono::NaiveDate,
) -> Result<String, String> {
    let (provider, route) = provider_route(provider).ok_or_else(unavailable)?;
    let region = canonical_region(region)?;
    let identifier = format!(
        "{}-{}-{}-{}-{}",
        provider,
        route,
        canonical_component(model, "model"),
        region,
        date.format("%Y-%m-%d")
    );
    validate_fixture_id(&identifier)?;
    Ok(identifier)
}

pub(super) fn validate_fixture_id(fixture_id: &str) -> Result<(), String> {
    let has_known_prefix = crate::services::reasoning_continuity::contract::RouteId::ALL
        .into_iter()
        .filter_map(|route| provider_route(route.provider_id()))
        .any(|(provider, route)| fixture_id.starts_with(&format!("{provider}-{route}-")));
    let mut pieces = fixture_id.rsplitn(4, '-');
    let date = [pieces.next(), pieces.next(), pieces.next()]
        .into_iter()
        .rev()
        .collect::<Option<Vec<_>>>()
        .map(|parts| parts.join("-"));
    let valid = fixture_id.len() <= MAX_FIXTURE_ID_BYTES
        && has_known_prefix
        && fixture_id.split('-').count() >= 5
        && !fixture_id.starts_with('-')
        && !fixture_id.ends_with('-')
        && !fixture_id.contains("--")
        && fixture_id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && date.is_some_and(|date| chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_ok());
    valid.then_some(()).ok_or_else(unavailable)
}

pub(super) fn is_report_name(name: &str) -> bool {
    name.strip_suffix(".json")
        .is_some_and(|id| validate_fixture_id(id).is_ok())
}

fn provider_route(provider: &str) -> Option<(&'static str, &'static str)> {
    Some(match provider {
        "ollama" => ("ollama", "local"),
        "google" => ("google", "api"),
        "mistral" => ("mistral", "api"),
        "cerebras" => ("cerebras", "api"),
        "openrouter" => ("openrouter", "api"),
        "openai" => ("openai", "api"),
        "deepseek" => ("deepseek", "api"),
        "xai" => ("xai", "api"),
        "xai-oauth" => ("xai", "oauth"),
        "moonshot" => ("moonshot", "api"),
        "moonshot-oauth" => ("moonshot", "oauth"),
        "zai" => ("zai", "api"),
        "codex-oauth" => ("codex", "oauth"),
        _ => return None,
    })
}

fn canonical_region(region: &str) -> Result<&str, String> {
    (!region.is_empty()
        && region.len() <= 32
        && region
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !region.starts_with('-')
        && !region.ends_with('-')
        && !region.contains("--"))
    .then_some(region)
    .ok_or_else(unavailable)
}

fn canonical_component(value: &str, fallback: &str) -> String {
    let mut result = String::with_capacity(value.len().min(96));
    let mut previous_was_separator = false;
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() {
            result.push(byte.to_ascii_lowercase() as char);
            previous_was_separator = false;
        } else if !previous_was_separator && !result.is_empty() {
            result.push('-');
            previous_was_separator = true;
        }
        if result.len() >= 96 {
            break;
        }
    }
    let result = result.trim_end_matches('-');
    if result.is_empty() {
        fallback.to_owned()
    } else {
        result.to_owned()
    }
}

fn unavailable() -> String {
    "Rapport de fixture indisponible".to_string()
}
