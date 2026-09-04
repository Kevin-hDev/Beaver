#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QualifiedContributionId {
    pub extension_id: String,
    pub local_id: String,
}

#[derive(Debug)]
pub(crate) struct LoadedResource {
    pub name: String,
    pub extension_id: String,
    pub bytes: Vec<u8>,
    pub signature: crate::services::file_signature::FileSignature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResourceLoadError {
    InvalidId,
    TooLarge,
    Unavailable,
    NotFound,
}

pub(crate) fn parse_qualified_contribution_id(value: &str) -> Result<QualifiedContributionId, ()> {
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

pub(crate) async fn load_for_session(
    resource_id: &str,
    session_id: &str,
) -> Result<LoadedResource, ResourceLoadError> {
    let records = super::list().map_err(|_| ResourceLoadError::Unavailable)?;
    let plugins = super::indexed_plugins();
    load_for_session_with(resource_id, session_id, &records, &plugins).await
}

pub(super) async fn load_for_session_with(
    resource_id: &str,
    session_id: &str,
    records: &[super::types::ExtensionRecord],
    plugins: &[super::registry_index::IndexedPlugin],
) -> Result<LoadedResource, ResourceLoadError> {
    let qualified =
        parse_qualified_contribution_id(resource_id).map_err(|_| ResourceLoadError::InvalidId)?;
    authorize_session(session_id, &qualified.extension_id).await?;
    let record = records
        .iter()
        .find(|record| active_approved_record(record, &qualified.extension_id))
        .ok_or(ResourceLoadError::Unavailable)?;
    let plugin = plugins
        .iter()
        .find(|plugin| plugin.id == qualified.extension_id)
        .ok_or(ResourceLoadError::Unavailable)?;
    let resource = plugin
        .resources
        .iter()
        .find(|resource| resource.id == qualified.local_id)
        .cloned()
        .ok_or(ResourceLoadError::NotFound)?;
    let root = std::path::PathBuf::from(&record.source);
    let path = resource.path.clone();
    tokio::task::spawn_blocking(move || {
        let loaded =
            super::read_verified_file(&root, &path, super::types::MAX_RESOURCE_FILE_BYTES as u64)
                .map_err(|error| match error {
                super::verified_file_read::FileReadError::Limit => ResourceLoadError::TooLarge,
                super::verified_file_read::FileReadError::NotFound => ResourceLoadError::NotFound,
                _ => ResourceLoadError::Unavailable,
            })?;
        if loaded.signature == crate::services::file_signature::FileSignature::Utf8
            && loaded.bytes.len() > super::types::MAX_TEXT_RESOURCE_BYTES
        {
            return Err(ResourceLoadError::TooLarge);
        }
        Ok(LoadedResource {
            name: resource.name,
            extension_id: qualified.extension_id,
            bytes: loaded.bytes,
            signature: loaded.signature,
        })
    })
    .await
    .map_err(|_| ResourceLoadError::Unavailable)?
}

pub(crate) async fn load_skill_for_session(
    skill_id: &str,
    session_id: &str,
) -> Result<LoadedResource, ResourceLoadError> {
    let records = super::list().map_err(|_| ResourceLoadError::Unavailable)?;
    let plugins = super::indexed_plugins();
    load_skill_for_session_with(skill_id, session_id, &records, &plugins).await
}

pub(super) async fn load_skill_for_session_with(
    skill_id: &str,
    session_id: &str,
    records: &[super::types::ExtensionRecord],
    plugins: &[super::registry_index::IndexedPlugin],
) -> Result<LoadedResource, ResourceLoadError> {
    let qualified =
        parse_qualified_contribution_id(skill_id).map_err(|_| ResourceLoadError::InvalidId)?;
    authorize_session(session_id, &qualified.extension_id).await?;
    let record = records
        .iter()
        .find(|record| active_approved_record(record, &qualified.extension_id))
        .ok_or(ResourceLoadError::Unavailable)?;
    let plugin = plugins
        .iter()
        .find(|plugin| plugin.id == qualified.extension_id)
        .ok_or(ResourceLoadError::Unavailable)?;
    let skill = plugin
        .skills
        .iter()
        .find(|skill| skill.id == qualified.local_id)
        .cloned()
        .ok_or(ResourceLoadError::NotFound)?;
    let root = std::path::PathBuf::from(&record.source);
    let path = skill.path.clone();
    tokio::task::spawn_blocking(move || {
        let loaded = super::read_verified_file(
            &root,
            &path,
            crate::services::skill_manifest_policy::MAX_SKILL_MANIFEST_BYTES as u64,
        )
        .map_err(|error| match error {
            super::verified_file_read::FileReadError::Limit => ResourceLoadError::TooLarge,
            super::verified_file_read::FileReadError::NotFound => ResourceLoadError::NotFound,
            _ => ResourceLoadError::Unavailable,
        })?;
        if loaded.signature != crate::services::file_signature::FileSignature::Utf8 {
            return Err(ResourceLoadError::Unavailable);
        }
        Ok(LoadedResource {
            name: skill.name,
            extension_id: qualified.extension_id,
            bytes: loaded.bytes,
            signature: loaded.signature,
        })
    })
    .await
    .map_err(|_| ResourceLoadError::Unavailable)?
}

async fn authorize_session(session_id: &str, extension_id: &str) -> Result<(), ResourceLoadError> {
    let session = crate::services::agent_local::session_store::get(session_id)
        .await
        .map_err(|_| ResourceLoadError::Unavailable)?;
    if session.parent_session_id.is_some() || session.subagent_type.is_some() {
        return Err(ResourceLoadError::Unavailable);
    }
    let state = crate::services::agent_local::extension_session_state::read(session_id)
        .await
        .map_err(|_| ResourceLoadError::Unavailable)?;
    let epoch = state.epoch.as_ref().ok_or(ResourceLoadError::Unavailable)?;
    let policy = crate::services::llm::route_profile::tool_policy(&epoch.provider, &epoch.model)
        .ok_or(ResourceLoadError::Unavailable)?;
    if policy.extensions != crate::services::llm::route_profile::ExtensionToolPolicy::All
        || !state
            .discovered_plugin_ids
            .iter()
            .any(|discovered| discovered == extension_id)
    {
        return Err(ResourceLoadError::Unavailable);
    }
    Ok(())
}

fn active_approved_record(record: &super::types::ExtensionRecord, extension_id: &str) -> bool {
    record.manifest.id == extension_id
        && record.enabled
        && record.trusted
        && record.status == super::types::ExtensionStatus::Active
}
