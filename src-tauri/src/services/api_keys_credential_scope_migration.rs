#[cfg(any(not(feature = "e2e"), test))]
#[derive(Default)]
struct ScopeMigrationReport {
    changed: Vec<RouteId>,
    blocked: Vec<RouteId>,
}

#[cfg(any(not(feature = "e2e"), test))]
fn migrate_llm_oauth_scope(
    map: &mut HashMap<String, String>,
    route: RouteId,
    logical_key: &str,
    report: &mut ScopeMigrationReport,
) {
    let Ok(physical_key) = prefixed_raw_key(logical_key) else {
        report.blocked.push(route);
        return;
    };
    let Some(json) = map.get(&physical_key).cloned().map(Zeroizing::new) else {
        return;
    };
    let Ok(mut record) = decode_llm_oauth_record(&json, route) else {
        report.blocked.push(route);
        return;
    };
    if record.schema_version == OAUTH_CREDENTIAL_SCHEMA_VERSION {
        return;
    }
    let Ok(scope) = generate_credential_scope() else {
        report.blocked.push(route);
        return;
    };
    record.schema_version = OAUTH_CREDENTIAL_SCHEMA_VERSION;
    record.credential_scope = Some(scope);
    match encode_llm_oauth_record(&record, route) {
        Ok(encoded) => {
            if stage_raw_entries(map, &[(logical_key, &encoded)]).is_ok() {
                report.changed.push(route);
            } else {
                report.blocked.push(route);
            }
        }
        Err(_) => report.blocked.push(route),
    }
}

#[cfg(any(not(feature = "e2e"), test))]
fn migrate_codex_oauth_scope(map: &mut HashMap<String, String>, report: &mut ScopeMigrationReport) {
    let route = RouteId::CodexOauth;
    let Ok(physical_key) = prefixed_raw_key(CODEX_OAUTH_KEY) else {
        report.blocked.push(route);
        return;
    };
    let Some(json) = map.get(&physical_key).cloned().map(Zeroizing::new) else {
        return;
    };
    let Ok(mut record) = decode_codex_oauth_record(&json) else {
        report.blocked.push(route);
        return;
    };
    if record.schema_version == OAUTH_CREDENTIAL_SCHEMA_VERSION {
        return;
    }
    let Ok(scope) = generate_credential_scope() else {
        report.blocked.push(route);
        return;
    };
    record.schema_version = OAUTH_CREDENTIAL_SCHEMA_VERSION;
    record.credential_scope = Some(scope);
    match encode_codex_oauth_record(&record) {
        Ok(encoded) => {
            if stage_raw_entries(map, &[(CODEX_OAUTH_KEY, &encoded)]).is_ok() {
                report.changed.push(route);
            } else {
                report.blocked.push(route);
            }
        }
        Err(_) => report.blocked.push(route),
    }
}
