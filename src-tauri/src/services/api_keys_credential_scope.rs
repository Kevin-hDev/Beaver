use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;

#[allow(dead_code, reason = "consumed by the session owner from Task 5")]
pub fn credential_scope(route: RouteId) -> Result<CredentialScope, String> {
    if route == RouteId::Ollama {
        return Ok(CredentialScope::local_uncredentialed());
    }
    if route == RouteId::Groq {
        return Err(scope_unavailable());
    }
    let state = STATE.lock().map_err(|_| scope_unavailable())?;
    let current = state.as_ref().ok_or_else(scope_unavailable)?;
    scope_from_map(&current.keys, route)
}

pub(crate) fn generate_credential_scope() -> Result<CredentialScope, String> {
    let mut bytes = [0_u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let encoded = URL_SAFE_NO_PAD.encode(bytes);
    bytes.fill(0);
    CredentialScope::authenticated(encoded).map_err(|_| scope_unavailable())
}

fn api_route_for_provider(provider_id: &str) -> Option<RouteId> {
    API_ROUTES
        .into_iter()
        .find(|route| api_provider_for_route(*route) == Some(provider_id))
}

fn stage_api_key(
    candidate: &mut HashMap<String, String>,
    provider_id: &str,
    key: Option<&str>,
    scope: Option<&CredentialScope>,
) -> Result<(), String> {
    let route = api_route_for_provider(provider_id);
    match (key, route) {
        (Some(key), Some(route)) => {
            let scope = scope.ok_or_else(scope_unavailable)?;
            scope
                .validate_for_route(route)
                .map_err(|_| scope_unavailable())?;
            let logical = credential_scope_vault_key(route).ok_or_else(scope_unavailable)?;
            candidate.insert(provider_id.to_string(), key.to_string());
            candidate.insert(prefixed_raw_key(&logical)?, scope.as_str().to_string());
        }
        (Some(key), None) if scope.is_none() => {
            candidate.insert(provider_id.to_string(), key.to_string());
        }
        (None, Some(route)) => {
            candidate.remove(provider_id);
            let logical = credential_scope_vault_key(route).ok_or_else(scope_unavailable)?;
            candidate.remove(&prefixed_raw_key(&logical)?);
        }
        (None, None) => {
            candidate.remove(provider_id);
        }
        _ => return Err(scope_unavailable()),
    }
    Ok(())
}

fn credential_scope_vault_key(route: RouteId) -> Option<String> {
    api_provider_for_route(route).map(|provider| format!("reasoning_scope:{provider}"))
}

#[allow(dead_code, reason = "supports credential_scope before Task 5 adopts it")]
fn scope_from_map<V>(map: &HashMap<String, V>, route: RouteId) -> Result<CredentialScope, String>
where
    V: AsRef<str>,
{
    if let Some(logical) = credential_scope_vault_key(route) {
        let physical = prefixed_raw_key(&logical)?;
        let value = map.get(&physical).ok_or_else(scope_unavailable)?;
        return CredentialScope::authenticated(value.as_ref().to_string())
            .map_err(|_| scope_unavailable());
    }
    let logical = oauth_vault_key(route).ok_or_else(scope_unavailable)?;
    let physical = prefixed_raw_key(logical)?;
    let json = map.get(&physical).ok_or_else(scope_unavailable)?.as_ref();
    match route {
        RouteId::XaiOauth | RouteId::MoonshotOauth => {
            decode_llm_oauth_record(json, route)?
                .credential_scope
                .clone()
                .ok_or_else(scope_unavailable)
        }
        RouteId::CodexOauth => decode_codex_oauth_record(json)?
            .credential_scope
            .clone()
            .ok_or_else(scope_unavailable),
        _ => Err(scope_unavailable()),
    }
}

fn commit_credential_scope_migration_with<P>(
    map: &mut HashMap<String, String>,
    persist: P,
) -> ScopeMigrationReport
where
    P: FnOnce(&HashMap<String, String>) -> Result<(), String>,
{
    let mut candidate = ZeroizingMap(map.clone());
    let mut report = prepare_credential_scope_migration(&mut candidate.0);
    if report.changed.is_empty() {
        return report;
    }
    if persist(&candidate.0).is_err() {
        report.blocked.append(&mut report.changed);
        return report;
    }
    std::mem::swap(map, &mut candidate.0);
    report
}

fn prepare_credential_scope_migration(map: &mut HashMap<String, String>) -> ScopeMigrationReport {
    let mut report = ScopeMigrationReport::default();
    for route in API_ROUTES {
        migrate_api_scope(map, route, &mut report);
    }
    migrate_llm_oauth_scope(map, RouteId::XaiOauth, LLM_OAUTH_XAI_KEY, &mut report);
    migrate_llm_oauth_scope(map, RouteId::MoonshotOauth, LLM_OAUTH_KIMI_KEY, &mut report);
    migrate_codex_oauth_scope(map, &mut report);
    report
}

fn migrate_api_scope(
    map: &mut HashMap<String, String>,
    route: RouteId,
    report: &mut ScopeMigrationReport,
) {
    let Some(provider) = api_provider_for_route(route) else {
        return;
    };
    if !map.contains_key(provider) {
        return;
    }
    let Some(logical) = credential_scope_vault_key(route) else {
        report.blocked.push(route);
        return;
    };
    let Ok(physical) = prefixed_raw_key(&logical) else {
        report.blocked.push(route);
        return;
    };
    if let Some(existing) = map.get(&physical) {
        if CredentialScope::authenticated(existing.clone()).is_err() {
            report.blocked.push(route);
        }
        return;
    }
    match generate_credential_scope() {
        Ok(scope) => {
            map.insert(physical, scope.as_str().to_string());
            report.changed.push(route);
        }
        Err(_) => report.blocked.push(route),
    }
}

const API_ROUTES: [RouteId; 9] = [
    RouteId::Google,
    RouteId::Mistral,
    RouteId::Cerebras,
    RouteId::OpenRouter,
    RouteId::OpenAi,
    RouteId::DeepSeek,
    RouteId::Xai,
    RouteId::Moonshot,
    RouteId::Zai,
];

fn api_provider_for_route(route: RouteId) -> Option<&'static str> {
    match route {
        RouteId::Google => Some("google"),
        RouteId::Mistral => Some("mistral"),
        RouteId::Cerebras => Some("cerebras"),
        RouteId::OpenRouter => Some("openrouter"),
        RouteId::OpenAi => Some("openai"),
        RouteId::DeepSeek => Some("deepseek"),
        RouteId::Xai => Some("xai"),
        RouteId::Moonshot => Some("moonshot"),
        RouteId::Zai => Some("zai"),
        _ => None,
    }
}

#[allow(dead_code, reason = "supports credential_scope before Task 5 adopts it")]
fn oauth_vault_key(route: RouteId) -> Option<&'static str> {
    match route {
        RouteId::XaiOauth => Some(LLM_OAUTH_XAI_KEY),
        RouteId::MoonshotOauth => Some(LLM_OAUTH_KIMI_KEY),
        RouteId::CodexOauth => Some(CODEX_OAUTH_KEY),
        _ => None,
    }
}

fn credential_scope_route_label(route: RouteId) -> &'static str {
    api_provider_for_route(route).unwrap_or(match route {
        RouteId::XaiOauth => "xai-oauth",
        RouteId::MoonshotOauth => "moonshot-oauth",
        RouteId::CodexOauth => "codex-oauth",
        _ => "unknown",
    })
}

fn scope_unavailable() -> String {
    "provider_configuration_invalid".to_string()
}
