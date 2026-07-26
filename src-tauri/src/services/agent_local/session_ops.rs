use super::session_store::{get, list, lock_session, save, validate_session_id};

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

pub async fn truncate_and_replace(
    session_id: &str,
    message_id: &str,
    replacement: Option<crate::services::agent_local::types_session::AgentMessage>,
) -> Result<(), String> {
    validate_session_id(session_id)?;
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
        session.accumulated_tokens =
            crate::services::token_counting::estimate_agent_messages_tokens(&session.messages);
        save(&session).await?;
    }
    Ok(())
}

pub async fn clear_project_id(project_id: &str) -> Result<(), String> {
    let all = list().await?;
    for meta in all {
        if meta.project_id.as_deref() == Some(project_id) {
            let mut session = get(&meta.id).await?;
            session.project_id = None;
            save(&session).await?;
        }
    }
    Ok(())
}
