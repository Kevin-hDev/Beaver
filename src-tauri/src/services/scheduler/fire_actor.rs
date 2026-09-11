use crate::services::automations::actor_context::AutomationActorGuard;
use uuid::Uuid;

pub(super) async fn register(
    session_id: &str,
    request_id: &str,
    automation_id: Uuid,
) -> Result<AutomationActorGuard, ()> {
    let session = crate::services::agent_local::session_store::get(session_id)
        .await
        .map_err(|_| ())?;
    let mut actor = crate::services::automations::actor_context::actor_for(
        session_id,
        session.is_gateway,
        session.gateway_channel_key.as_deref(),
        None,
    )
    .map_err(|_| ())?;
    actor.current_automation_id = Some(automation_id);
    crate::services::automations::actor_context::register_actor(request_id, actor).map_err(|_| ())
}
