#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QualifiedContributionId {
    pub extension_id: String,
    pub local_id: String,
}

pub(crate) fn parse(value: &str) -> Result<QualifiedContributionId, ()> {
    let mut segments = value.split(':');
    let prefix = segments.next();
    let extension_id = segments.next();
    let local_id = segments.next();
    if prefix != Some("extension") || segments.next().is_some() {
        return Err(());
    }
    let (Some(extension_id), Some(local_id)) = (extension_id, local_id) else {
        return Err(());
    };
    if super::validation::identifier(extension_id).is_err()
        || super::validation::identifier(local_id).is_err()
    {
        return Err(());
    }
    Ok(QualifiedContributionId {
        extension_id: extension_id.to_string(),
        local_id: local_id.to_string(),
    })
}
