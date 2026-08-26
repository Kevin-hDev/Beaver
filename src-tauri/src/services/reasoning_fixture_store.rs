use std::path::{Path, PathBuf};

const MAX_REPORTS: usize = 64;
const MAX_FIXTURE_ID_BYTES: usize = 240;
#[cfg(debug_assertions)]
const MAX_REPORT_BYTES: usize = 256 * 1024;

#[cfg(debug_assertions)]
pub async fn write_report(
    session_id: &str,
    fixture_id: &str,
    report: Vec<u8>,
) -> Result<(), String> {
    crate::services::agent_local::session_id::validate_session_id(session_id)?;
    validate_fixture_id(fixture_id)?;
    if report.is_empty() || report.len() > MAX_REPORT_BYTES {
        return Err(unavailable());
    }
    let root = crate::services::paths::data_dir().join("reasoning-fixture-reports");
    let session_id = session_id.to_string();
    let fixture_id = fixture_id.to_string();
    tokio::task::spawn_blocking(move || write_at(&root, &session_id, &fixture_id, &report))
        .await
        .map_err(|_| unavailable())?
}

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

pub async fn remove_for_session(session_id: &str) -> Result<(), String> {
    crate::services::agent_local::session_id::validate_session_id(session_id)?;
    let path = crate::services::paths::data_dir()
        .join("reasoning-fixture-reports")
        .join(session_id);
    tokio::task::spawn_blocking(move || remove_private_dir(&path))
        .await
        .map_err(|_| unavailable())?
}

#[cfg(any(debug_assertions, test))]
fn write_at(root: &Path, session_id: &str, fixture_id: &str, report: &[u8]) -> Result<(), String> {
    let directory = root.join(session_id);
    crate::services::private_store::ensure_private_dir(&directory).map_err(|_| unavailable())?;
    prune(&directory)?;
    crate::services::private_store::atomic_write(
        &directory.join(format!("{fixture_id}.json")),
        report,
    )
    .map_err(|_| unavailable())
}

#[cfg(any(debug_assertions, test))]
fn prune(directory: &Path) -> Result<(), String> {
    let mut reports = valid_reports(directory)?;
    if reports.len() < MAX_REPORTS {
        return Ok(());
    }
    reports.sort_by_key(|entry| entry.1);
    let path = reports
        .first()
        .map(|entry| entry.0.clone())
        .ok_or_else(unavailable)?;
    std::fs::remove_file(path).map_err(|_| unavailable())
}

fn remove_private_dir(path: &Path) -> Result<(), String> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err(unavailable()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(unavailable());
    }
    for (entry, _) in valid_reports(path)? {
        std::fs::remove_file(entry).map_err(|_| unavailable())?;
    }
    std::fs::remove_dir(path).map_err(|_| unavailable())
}

fn valid_reports(directory: &Path) -> Result<Vec<(PathBuf, std::time::SystemTime)>, String> {
    let mut reports = Vec::new();
    for entry in std::fs::read_dir(directory).map_err(|_| unavailable())? {
        let entry = entry.map_err(|_| unavailable())?;
        if reports.len() >= MAX_REPORTS {
            return Err(unavailable());
        }
        let file_type = entry.file_type().map_err(|_| unavailable())?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(unavailable)?;
        if file_type.is_symlink() || !file_type.is_file() || !is_report_name(name) {
            return Err(unavailable());
        }
        let modified = entry
            .metadata()
            .map_err(|_| unavailable())?
            .modified()
            .map_err(|_| unavailable())?;
        reports.push((entry.path(), modified));
    }
    Ok(reports)
}

fn validate_fixture_id(fixture_id: &str) -> Result<(), String> {
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

fn is_report_name(name: &str) -> bool {
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
        "groq" => ("groq", "api"),
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
    (!result.is_empty())
        .then_some(result.to_owned())
        .unwrap_or_else(|| fallback.to_owned())
}

fn unavailable() -> String {
    "Rapport de fixture indisponible".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_evicts_the_oldest_report_without_following_links() {
        let temporary = tempfile::tempdir().unwrap();
        let directory = temporary.path().join("reports");
        let session = "11111111-1111-4111-8111-111111111111";
        let date = chrono::NaiveDate::from_ymd_opt(2026, 8, 26).unwrap();
        for index in 0..MAX_REPORTS {
            write_at(
                &directory,
                session,
                &derive_fixture_id("ollama", &format!("fixture-{index}"), "test", date).unwrap(),
                b"{}",
            )
            .unwrap();
        }
        write_at(
            &directory,
            session,
            &derive_fixture_id("ollama", "fixture-overflow", "test", date).unwrap(),
            b"{}",
        )
        .unwrap();
        assert_eq!(
            valid_reports(&directory.join(session)).unwrap().len(),
            MAX_REPORTS
        );
    }

    #[test]
    fn rejects_noncanonical_report_names() {
        assert!(!is_report_name("report.json"));
        assert!(!is_report_name("../report.json"));
        assert!(!is_report_name("11111111-1111-4111-8111-111111111111.json"));
    }

    #[test]
    fn derives_a_bounded_canonical_report_name() {
        let id = derive_fixture_id(
            "xai-oauth",
            "Grok 4.6/preview",
            "eu-west-1",
            chrono::NaiveDate::from_ymd_opt(2026, 8, 26).unwrap(),
        )
        .unwrap();
        assert_eq!(id, "xai-oauth-grok-4-6-preview-eu-west-1-2026-08-26");
        assert!(validate_fixture_id(&id).is_ok());
        assert!(
            derive_fixture_id("xai", "model", "../escape", chrono::Utc::now().date_naive())
                .is_err()
        );
    }
}
