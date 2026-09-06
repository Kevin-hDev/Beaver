// Deduplication uses the submitted source. Update origins are resolved by the sole
// worker; identity validation before publication rejects cross-kind duplicates.
use super::{limits::INVALID, InstallKind, InstallRequest};

pub(super) fn normalize(
    request: InstallRequest,
) -> Result<(InstallRequest, String, InstallKind), String> {
    match request {
        InstallRequest::Local { path } => {
            if std::path::Path::new(&path)
                .components()
                .any(|part| part == std::path::Component::ParentDir)
                || path.chars().any(char::is_control)
            {
                return Err(INVALID.into());
            }
            super::super::validation::source_input(&path).map_err(|_| INVALID)?;
            let path = std::path::Path::new(&path)
                .canonicalize()
                .map_err(|_| INVALID)?;
            let path = path.to_str().ok_or(INVALID)?.to_owned();
            Ok((
                InstallRequest::Local { path: path.clone() },
                format!("local:{path}"),
                InstallKind::Local,
            ))
        }
        InstallRequest::Git { locator } => {
            let source = super::super::source_validation::git(&locator).map_err(|_| INVALID)?;
            let locator = format!(
                "{}{}",
                source.clone_url,
                source
                    .reference
                    .map(|r| format!("#{r}"))
                    .unwrap_or_default()
            );
            Ok((
                InstallRequest::Git {
                    locator: locator.clone(),
                },
                format!("git:{locator}"),
                InstallKind::Git,
            ))
        }
        InstallRequest::Npm { locator } => {
            let source = super::super::source_validation::npm(&locator).map_err(|_| INVALID)?;
            Ok((
                InstallRequest::Npm {
                    locator: source.locator.clone(),
                },
                format!("npm:{}", source.locator),
                InstallKind::Npm,
            ))
        }
        InstallRequest::Update { extension_id } => {
            super::super::validation::identifier(&extension_id).map_err(|_| INVALID)?;
            Ok((
                InstallRequest::Update {
                    extension_id: extension_id.clone(),
                },
                format!("update:{extension_id}"),
                InstallKind::Update,
            ))
        }
    }
}

pub(super) fn id(value: &str) -> Result<(), String> {
    if value.len() != 36 || uuid::Uuid::parse_str(value).is_err() {
        return Err(INVALID.into());
    }
    Ok(())
}

pub(super) fn display_name(request: &InstallRequest) -> String {
    // Only the source's label is public, never its parent path, host or credentials.
    let label = match request {
        InstallRequest::Local { path } => std::path::Path::new(path)
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("Local")
            .to_owned(),
        InstallRequest::Git { locator } => super::super::source_validation::git(locator)
            .map(|source| {
                source
                    .clone_url
                    .rsplit(['/', ':'])
                    .next()
                    .unwrap_or("Git")
                    .trim_end_matches(".git")
                    .to_owned()
            })
            .unwrap_or_else(|_| "Git".into()),
        InstallRequest::Npm { locator } => super::super::source_validation::npm(locator)
            .map(|source| source.package_name)
            .unwrap_or_else(|_| "npm".into()),
        InstallRequest::Update { extension_id } => extension_id.clone(),
    };
    label
        .chars()
        .filter(|value| !value.is_control())
        .take(super::super::types::MAX_EXTENSION_NAME_CHARS)
        .collect()
}
