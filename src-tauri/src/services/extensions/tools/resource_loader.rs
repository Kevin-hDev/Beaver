#[derive(Debug)]
pub(crate) struct LoadedResource {
    pub name: String,
    pub extension_id: String,
    pub qualified_resource_id: String,
    pub catalog_fingerprint: String,
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

pub(crate) async fn load_resource_for_history(
    resource_id: &str,
) -> Result<LoadedResource, ResourceLoadError> {
    let records = super::list().map_err(|_| ResourceLoadError::Unavailable)?;
    let (plugins, catalog_fingerprint) =
        super::registry_index::indexed_plugins_with_catalog_version()
            .map_err(|_| ResourceLoadError::Unavailable)?;
    load_current_with(resource_id, &records, &plugins, &catalog_fingerprint).await
}

#[cfg(test)]
pub(super) async fn load_for_session_with(
    resource_id: &str,
    session_id: &str,
    records: &[super::types::ExtensionRecord],
    plugins: &[super::registry_index::IndexedPlugin],
    catalog_fingerprint: &str,
) -> Result<LoadedResource, ResourceLoadError> {
    let qualified =
        super::resource_identifier::parse(resource_id).map_err(|_| ResourceLoadError::InvalidId)?;
    authorize_session(session_id, &qualified.extension_id).await?;
    load_current_with_qualified(qualified, records, plugins, catalog_fingerprint).await
}

async fn load_current_with(
    resource_id: &str,
    records: &[super::types::ExtensionRecord],
    plugins: &[super::registry_index::IndexedPlugin],
    catalog_fingerprint: &str,
) -> Result<LoadedResource, ResourceLoadError> {
    let qualified =
        super::resource_identifier::parse(resource_id).map_err(|_| ResourceLoadError::InvalidId)?;
    load_current_with_qualified(qualified, records, plugins, catalog_fingerprint).await
}

async fn load_current_with_qualified(
    qualified: super::resource_identifier::QualifiedContributionId,
    records: &[super::types::ExtensionRecord],
    plugins: &[super::registry_index::IndexedPlugin],
    catalog_fingerprint: &str,
) -> Result<LoadedResource, ResourceLoadError> {
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
    let qualified_resource_id = format!(
        "extension:{}:{}",
        qualified.extension_id, qualified.local_id
    );
    let catalog_fingerprint = catalog_fingerprint.to_owned();
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
            qualified_resource_id,
            catalog_fingerprint,
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
    let (plugins, catalog_fingerprint) =
        super::registry_index::indexed_plugins_with_catalog_version()
            .map_err(|_| ResourceLoadError::Unavailable)?;
    load_skill_for_session_with(
        skill_id,
        session_id,
        &records,
        &plugins,
        &catalog_fingerprint,
    )
    .await
}
pub(super) async fn load_skill_for_session_with(
    skill_id: &str,
    session_id: &str,
    records: &[super::types::ExtensionRecord],
    plugins: &[super::registry_index::IndexedPlugin],
    catalog_fingerprint: &str,
) -> Result<LoadedResource, ResourceLoadError> {
    let qualified =
        super::resource_identifier::parse(skill_id).map_err(|_| ResourceLoadError::InvalidId)?;
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
    let qualified_resource_id = format!(
        "extension:{}:{}",
        qualified.extension_id, qualified.local_id
    );
    let catalog_fingerprint = catalog_fingerprint.to_owned();
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
            qualified_resource_id,
            catalog_fingerprint,
            bytes: loaded.bytes,
            signature: loaded.signature,
        })
    })
    .await
    .map_err(|_| ResourceLoadError::Unavailable)?
}

pub(super) async fn authorize_session(
    session_id: &str,
    extension_id: &str,
) -> Result<(), ResourceLoadError> {
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

pub(super) fn active_approved_record(
    record: &super::types::ExtensionRecord,
    extension_id: &str,
) -> bool {
    record.manifest.id == extension_id
        && record.enabled
        && record.trusted
        && record.status == super::types::ExtensionStatus::Active
}
