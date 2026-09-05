#[derive(Debug, Clone)]
pub(crate) struct PreparedResource {
    pub session_id: String,
    pub name: String,
    pub extension_id: String,
    pub qualified_resource_id: String,
    pub catalog_fingerprint: String,
    pub root: std::path::PathBuf,
    pub relative_path: String,
}

pub(crate) async fn prepare_for_session(
    resource_id: &str,
    session_id: &str,
) -> Result<PreparedResource, super::resource_loader::ResourceLoadError> {
    let records =
        super::list().map_err(|_| super::resource_loader::ResourceLoadError::Unavailable)?;
    let (plugins, catalog_fingerprint) =
        super::registry_index::indexed_plugins_with_catalog_version()
            .map_err(|_| super::resource_loader::ResourceLoadError::Unavailable)?;
    prepare_for_session_with(
        resource_id,
        session_id,
        &records,
        &plugins,
        &catalog_fingerprint,
    )
    .await
}

pub(super) async fn prepare_for_session_with(
    resource_id: &str,
    session_id: &str,
    records: &[super::types::ExtensionRecord],
    plugins: &[super::registry_index::IndexedPlugin],
    catalog_fingerprint: &str,
) -> Result<PreparedResource, super::resource_loader::ResourceLoadError> {
    let qualified = super::resource_identifier::parse(resource_id)
        .map_err(|_| super::resource_loader::ResourceLoadError::InvalidId)?;
    super::resource_loader::authorize_session(session_id, &qualified.extension_id).await?;
    let record = records
        .iter()
        .find(|record| {
            super::resource_loader::active_approved_record(record, &qualified.extension_id)
        })
        .ok_or(super::resource_loader::ResourceLoadError::Unavailable)?;
    let plugin = plugins
        .iter()
        .find(|plugin| plugin.id == qualified.extension_id)
        .ok_or(super::resource_loader::ResourceLoadError::Unavailable)?;
    let resource = plugin
        .resources
        .iter()
        .find(|resource| resource.id == qualified.local_id)
        .cloned()
        .ok_or(super::resource_loader::ResourceLoadError::NotFound)?;
    Ok(PreparedResource {
        session_id: session_id.to_owned(),
        name: resource.name,
        extension_id: qualified.extension_id.clone(),
        qualified_resource_id: format!(
            "extension:{}:{}",
            qualified.extension_id, qualified.local_id
        ),
        catalog_fingerprint: catalog_fingerprint.to_owned(),
        root: std::path::PathBuf::from(&record.source),
        relative_path: resource.path,
    })
}

pub(crate) async fn revalidate_for_resolution(
    session_id: &str,
    extension_id: &str,
    qualified_resource_id: &str,
    catalog_fingerprint: &str,
    root: &std::path::Path,
    relative_path: &str,
) -> Result<(), super::resource_loader::ResourceLoadError> {
    let records =
        super::list().map_err(|_| super::resource_loader::ResourceLoadError::Unavailable)?;
    let (plugins, current_fingerprint) =
        super::registry_index::indexed_plugins_with_catalog_version()
            .map_err(|_| super::resource_loader::ResourceLoadError::Unavailable)?;
    revalidate_for_resolution_with(
        session_id,
        extension_id,
        qualified_resource_id,
        catalog_fingerprint,
        root,
        relative_path,
        &records,
        &plugins,
        &current_fingerprint,
    )
    .await
}

#[expect(
    clippy::too_many_arguments,
    reason = "revalidation compares one deferred resource to one current authority snapshot"
)]
pub(super) async fn revalidate_for_resolution_with(
    session_id: &str,
    extension_id: &str,
    qualified_resource_id: &str,
    catalog_fingerprint: &str,
    root: &std::path::Path,
    relative_path: &str,
    records: &[super::types::ExtensionRecord],
    plugins: &[super::registry_index::IndexedPlugin],
    current_fingerprint: &str,
) -> Result<(), super::resource_loader::ResourceLoadError> {
    let current = prepare_for_session_with(
        qualified_resource_id,
        session_id,
        records,
        plugins,
        current_fingerprint,
    )
    .await?;
    (current.extension_id == extension_id
        && current.qualified_resource_id == qualified_resource_id
        && current.catalog_fingerprint == catalog_fingerprint
        && current.root == root
        && current.relative_path == relative_path)
        .then_some(())
        .ok_or(super::resource_loader::ResourceLoadError::Unavailable)
}
