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
