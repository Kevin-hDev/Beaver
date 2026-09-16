use super::*;
use chrono::Utc;
use uuid::Uuid;

pub(crate) async fn list(
    actor: &AutomationActor,
    owner: &ExtensionActorIdentity,
) -> Result<Vec<crate::models::AutomationDefinition>, AutomationError> {
    let root = root();
    let operation = super::audit_store::begin(&root, actor, "list", None, Vec::new()).await?;
    let result = super::service_owned::list_at(&root, owner).await;
    let count = result.as_ref().ok().map(Vec::len);
    super::audit_store::complete(&root, actor, operation, "list", None, result, count).await
}

pub(crate) async fn create(
    actor: &AutomationActor,
    owner: &ExtensionActorIdentity,
    input: CreateAutomation,
) -> Result<AutomationDetail, AutomationError> {
    let root = root();
    let operation = super::audit_store::begin(
        &root,
        actor,
        "create",
        None,
        ["name", "description", "prompt", "schedule"]
            .into_iter()
            .map(str::to_string)
            .collect(),
    )
    .await?;
    let result = super::service_owned::create_at(&root, actor, owner, input, Utc::now()).await;
    let id = result.as_ref().ok().map(|detail| detail.definition.id);
    let result =
        super::audit_store::complete(&root, actor, operation, "create", id, result, None).await?;
    crate::services::scheduler::notify_config_changed();
    Ok(result)
}

pub(crate) async fn update(
    actor: &AutomationActor,
    owner: &ExtensionActorIdentity,
    id: Uuid,
    revision: u64,
    patch: UpdateAutomation,
) -> Result<AutomationDetail, AutomationError> {
    let root = root();
    let fields = super::service_helpers::changed_fields(&patch);
    let operation = super::audit_store::begin(&root, actor, "update", Some(id), fields).await?;
    let result =
        super::service_owned::update_at(&root, owner, id, revision, patch, Utc::now()).await;
    let result =
        super::audit_store::complete(&root, actor, operation, "update", Some(id), result, None)
            .await?;
    crate::services::scheduler::notify_config_changed();
    Ok(result)
}

pub(crate) async fn set_active(
    actor: &AutomationActor,
    owner: &ExtensionActorIdentity,
    id: Uuid,
    revision: u64,
    active: bool,
) -> Result<AutomationDetail, AutomationError> {
    let root = root();
    let operation =
        super::audit_store::begin(&root, actor, "update", Some(id), vec!["status".into()]).await?;
    let paused = crate::services::config::read_config()
        .map(|config| config.heartbeat.global_paused)
        .map_err(|_| AutomationError::StoreUnavailable)?;
    let result =
        super::service_owned::set_active_at(&root, owner, id, revision, active, paused, Utc::now())
            .await;
    let result =
        super::audit_store::complete(&root, actor, operation, "update", Some(id), result, None)
            .await?;
    crate::services::scheduler::notify_config_changed();
    Ok(result)
}

pub(crate) async fn delete(
    actor: &AutomationActor,
    owner: &ExtensionActorIdentity,
    id: Uuid,
    revision: u64,
) -> Result<(), AutomationError> {
    let root = root();
    let operation = super::audit_store::begin(&root, actor, "delete", Some(id), Vec::new()).await?;
    let result = super::service_owned::delete_at(&root, owner, id, revision).await;
    super::audit_store::complete(&root, actor, operation, "delete", Some(id), result, None).await?;
    crate::services::scheduler::notify_config_changed();
    Ok(())
}

fn root() -> std::path::PathBuf {
    crate::services::paths::data_dir()
}
