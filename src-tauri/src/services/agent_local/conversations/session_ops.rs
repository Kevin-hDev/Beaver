use super::session_store::{get, validate_session_id};
#[cfg(test)]
use super::session_store::{lock_session, save};

pub use super::session_mutations::{
    apply_metadata_patch, edit_user_message, set_compression_profile,
};

pub async fn export_markdown(id: &str) -> Result<String, String> {
    validate_session_id(id)?;
    let session = get(id).await?;
    let mut md = format!("# {}\n\n", session.name);
    for msg in &session.messages {
        let role = match msg.role.as_str() {
            "user" => "**Utilisateur**",
            "assistant" => "**Assistant**",
            "tool" => "**Outil**",
            _ => &msg.role,
        };
        md.push_str(&format!("### {role}\n\n{}\n\n---\n\n", msg.content));
    }
    Ok(md)
}

#[cfg(test)]
pub async fn truncate_and_replace(
    session_id: &str,
    message_id: &str,
    replacement: Option<crate::services::agent_local::types_session::AgentMessage>,
) -> Result<(), String> {
    validate_session_id(session_id)?;
    if let Some(message) = replacement.as_ref() {
        super::session_store_messages::validate_legacy_ipc_message(message)?;
    }
    let lock = lock_session(session_id).await;
    let _guard = lock.lock().await;
    let mut session = get(session_id).await?;
    if let Some(idx) = session.messages.iter().position(|m| m.id == message_id) {
        match replacement {
            Some(new_msg) => {
                session.messages.truncate(idx);
                session.messages.push(new_msg);
            }
            None => {
                session.messages.truncate(idx + 1);
            }
        }
        super::session_store_messages::recompute_accumulated_tokens(&mut session);
        save(&session).await?;
    }
    Ok(())
}

pub async fn clear_project_id(project_id: &str) -> Result<(), String> {
    clear_project_id_inner(project_id, || async {}).await
}

async fn clear_project_id_inner<F, Fut>(project_id: &str, after_list: F) -> Result<(), String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    // Les sessions archivées doivent être détachées elles aussi : elles peuvent
    // être restaurées après la suppression du projet.
    let all = super::session_index::read_index().await?;
    after_list().await;
    for meta in all {
        if meta.project_id.as_deref() == Some(project_id) {
            super::session_store_updates::update_locked(&meta.id, |session| {
                if session.project_id.as_deref() == Some(project_id) {
                    session.project_id = None;
                }
            })
            .await?;
        }
    }
    Ok(())
}

#[cfg(test)]
async fn clear_project_id_with_after_list<F, Fut>(
    project_id: &str,
    after_list: F,
) -> Result<(), String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    clear_project_id_inner(project_id, after_list).await
}

#[cfg(test)]
#[path = "session_ops_tests.rs"]
mod tests;
