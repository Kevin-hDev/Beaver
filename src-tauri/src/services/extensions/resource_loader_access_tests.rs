use super::resource_loader::ResourceLoadError;
use super::resource_loader_test_support::TestRegistry;

#[tokio::test]
async fn loading_requires_inspection_and_an_extension_capable_route() {
    let registry = TestRegistry::new().await;
    let session_id = registry.session("openai", "gpt-5.4").await;
    let records = registry.records();
    let plugins = registry.plugins(&records);

    assert_unavailable(&session_id, &records, &plugins).await;
    registry.inspect(&session_id).await;
    assert_eq!(load_reference(&session_id, &records, &plugins).await, b"first");

    registry
        .configure(&session_id, "openrouter", "groq/llama-3.3-70b")
        .await;
    assert_unavailable(&session_id, &records, &plugins).await;
    assert_eq!(
        super::resource_loader::load_skill_for_session_with(
            "extension:example.first:guide",
            &session_id,
            &records,
            &plugins,
        )
        .await
        .unwrap_err(),
        ResourceLoadError::Unavailable
    );

    registry
        .configure(&session_id, "openrouter", "groq/compound")
        .await;
    assert_unavailable(&session_id, &records, &plugins).await;

    crate::services::agent_local::session_store::delete_one(&session_id)
        .await
        .expect("delete session");
}

#[tokio::test]
async fn loading_refuses_subagents_missing_sessions_and_stale_indexes() {
    let registry = TestRegistry::new().await;
    let session_id = registry.session("openai", "gpt-5.4").await;
    let records = registry.records();
    let plugins = registry.plugins(&records);
    registry.inspect(&session_id).await;

    let parent = crate::services::agent_local::session_store::get(&session_id)
        .await
        .expect("parent session");
    let mut child = parent.clone();
    child.id = uuid::Uuid::new_v4().to_string();
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("coder".into());
    crate::services::agent_local::session_store::save(&child)
        .await
        .expect("child session");
    registry.configure(&child.id, "openai", "gpt-5.4").await;
    crate::services::agent_local::extension_session_state::mutate(&child.id, |state| {
        state.discovered_plugin_ids.push("example.first".into());
        Ok(())
    })
    .await
    .expect("child discovery fixture");

    assert_unavailable(&child.id, &records, &plugins).await;
    assert_unavailable(&uuid::Uuid::new_v4().to_string(), &records, &plugins).await;
    let reloaded_records = vec![registry.second.clone()];
    let reloaded_plugins = registry.plugins(&reloaded_records);
    assert_unavailable(&session_id, &reloaded_records, &reloaded_plugins).await;

    crate::services::agent_local::session_store::delete_one(&child.id)
        .await
        .expect("delete child");
    crate::services::agent_local::session_store::delete_one(&session_id)
        .await
        .expect("delete parent");
}

#[tokio::test]
async fn disabled_or_untrusted_records_fail_closed() {
    let registry = TestRegistry::new().await;
    let session_id = registry.session("openai", "gpt-5.4").await;
    let active_records = registry.records();
    let active_plugins = registry.plugins(&active_records);
    registry.inspect(&session_id).await;

    let mut disabled = registry.first.clone();
    disabled.enabled = false;
    let disabled_records = vec![disabled, registry.second.clone()];
    let disabled_plugins = registry.plugins(&disabled_records);
    assert_unavailable(&session_id, &disabled_records, &disabled_plugins).await;

    let mut untrusted = registry.first.clone();
    untrusted.trusted = false;
    let untrusted_records = vec![untrusted, registry.second.clone()];
    let untrusted_plugins = registry.plugins(&untrusted_records);
    assert_unavailable(&session_id, &untrusted_records, &untrusted_plugins).await;

    assert_eq!(
        load_reference(&session_id, &active_records, &active_plugins).await,
        b"first"
    );

    crate::services::agent_local::session_store::delete_one(&session_id)
        .await
        .expect("delete session");
}

async fn assert_unavailable(
    session_id: &str,
    records: &[super::types::ExtensionRecord],
    plugins: &[super::registry_index::IndexedPlugin],
) {
    assert_eq!(
        super::resource_loader::load_for_session_with(
            "extension:example.first:reference",
            session_id,
            records,
            plugins,
        )
        .await
        .unwrap_err(),
        ResourceLoadError::Unavailable
    );
}

async fn load_reference(
    session_id: &str,
    records: &[super::types::ExtensionRecord],
    plugins: &[super::registry_index::IndexedPlugin],
) -> Vec<u8> {
    super::resource_loader::load_for_session_with(
        "extension:example.first:reference",
        session_id,
        records,
        plugins,
    )
    .await
    .expect("reference resource")
    .bytes
}
