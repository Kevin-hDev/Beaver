use super::resource_loader::ResourceLoadError;
use super::resource_loader_test_support::TestRegistry;

#[tokio::test]
async fn real_signatures_and_limits_override_declared_resource_types() {
    let registry = TestRegistry::new().await;
    let session_id = registry.session("openai", "gpt-5.4").await;
    let records = registry.records();
    let plugins = registry.plugins(&records);
    registry.inspect(&session_id).await;

    let image = load("image-as-text", &session_id, &records, &plugins)
        .await
        .expect("image declared as text");
    assert_eq!(
        image.signature,
        crate::services::file_signature::FileSignature::Png
    );
    let unknown = load("unknown", &session_id, &records, &plugins)
        .await
        .expect("unknown binary");
    assert_eq!(
        unknown.signature,
        crate::services::file_signature::FileSignature::Binary
    );
    let exact = load("text-exact", &session_id, &records, &plugins)
        .await
        .expect("exact text limit");
    assert_eq!(exact.bytes.len(), super::types::MAX_TEXT_RESOURCE_BYTES);
    assert_eq!(
        load("text-too-large", &session_id, &records, &plugins)
            .await
            .unwrap_err(),
        ResourceLoadError::TooLarge
    );
    assert_eq!(
        load("missing", &session_id, &records, &plugins)
            .await
            .unwrap_err(),
        ResourceLoadError::NotFound
    );

    crate::services::agent_local::session_store::delete_one(&session_id)
        .await
        .expect("delete session");
}

#[tokio::test]
async fn local_ids_are_scoped_to_each_extension_and_rebuild_after_restart() {
    let registry = TestRegistry::new().await;
    let session_id = registry.session("openai", "gpt-5.4").await;
    let records = registry.records();
    let plugins = registry.plugins(&records);
    registry.inspect(&session_id).await;

    let first = super::resource_loader::load_skill_for_session_with(
        "extension:example.first:guide",
        &session_id,
        &records,
        &plugins,
    )
    .await
    .expect("first skill");
    let second = super::resource_loader::load_skill_for_session_with(
        "extension:example.second:guide",
        &session_id,
        &records,
        &plugins,
    )
    .await
    .expect("second skill");
    assert_ne!(first.bytes, second.bytes);

    let cleared = super::registry_state::reset_hosted_runtime(vec![
        registry.first.clone(),
        registry.second.clone(),
    ]);
    let cleared_plugins = registry.plugins(&cleared);
    assert_eq!(
        load("reference", &session_id, &cleared, &cleared_plugins)
            .await
            .unwrap_err(),
        ResourceLoadError::Unavailable
    );

    crate::services::agent_local::extension_session_state::remove(&session_id)
        .await
        .expect("clear discovery state");
    registry.configure(&session_id, "openai", "gpt-5.4").await;
    registry.inspect(&session_id).await;
    assert_eq!(
        load("reference", &session_id, &records, &plugins)
            .await
            .expect("resource after reconstructed index")
            .bytes,
        b"first"
    );

    crate::services::agent_local::session_store::delete_one(&session_id)
        .await
        .expect("delete session");
}

async fn load(
    local_id: &str,
    session_id: &str,
    records: &[super::types::ExtensionRecord],
    plugins: &[super::registry_index::IndexedPlugin],
) -> Result<super::resource_loader::LoadedResource, ResourceLoadError> {
    super::resource_loader::load_for_session_with(
        &format!("extension:example.first:{local_id}"),
        session_id,
        records,
        plugins,
    )
    .await
}
