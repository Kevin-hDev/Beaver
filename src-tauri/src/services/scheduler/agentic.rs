use crate::commands::agent_chat_task::{run_stream_task, StreamCapabilityHints, StreamTaskParams};
use crate::models::ScheduledWakeup;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::ChatMessage;
use std::collections::BTreeSet;
use std::path::PathBuf;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub async fn run(
    app: &AppHandle,
    wakeup: &ScheduledWakeup,
    session_id: &str,
) -> Result<(String, u32), String> {
    let working_dir = validate_working_dir(&wakeup.working_dir)?;
    let tools = selected_tools(&wakeup.tool_names).await?;
    let system = build_system(&wakeup.skill_ids).await?;
    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: system,
            ..Default::default()
        },
        ChatMessage {
            role: "user".into(),
            content: wakeup.prompt.clone(),
            ..Default::default()
        },
    ];
    let completed = run_stream_task(StreamTaskParams {
        on_event: AgentEventEmitter::new(app.clone(), session_id.to_string()),
        session_id: session_id.to_string(),
        request_id: Uuid::new_v4().to_string(),
        model: wakeup.model.clone(),
        messages,
        tools,
        think: false,
        provider: wakeup.provider.clone(),
        working_dir,
        outputs_dir: None,
        capability_hints: StreamCapabilityHints::default(),
        reasoning_mode: None,
        permission_mode_override: Some("subagent".into()),
        permission_emitter: None,
        parent_message_inbox: None,
        subagent_profile: None,
        plan_mode: Some(false),
        cancel: CancellationToken::new(),
    })
    .await?;
    let reply = completed
        .iter()
        .rev()
        .find(|message| message.role == "assistant" && !message.content.trim().is_empty())
        .map(|message| message.content.clone())
        .ok_or_else(|| "L'automatisation n'a produit aucun résultat.".to_string())?;
    let tokens =
        crate::services::token_counting::estimate_text_tokens(&reply).min(u32::MAX as usize) as u32;
    Ok((reply, tokens))
}

fn validate_working_dir(value: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(value);
    if !path.is_absolute() || !path.is_dir() {
        return Err("Dossier d'automatisation indisponible.".into());
    }
    path.canonicalize()
        .map_err(|_| "Dossier d'automatisation indisponible.".to_string())
}

async fn selected_tools(names: &[String]) -> Result<Vec<serde_json::Value>, String> {
    if names.len() > 12 {
        return Err("Configuration d'automatisation invalide.".into());
    }
    let requested = names.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let settings = crate::services::agent_local::agent_settings::load().await;
    let definitions = crate::services::agent_local::tool_catalog::filter_tool_definitions(
        crate::services::agent_local::tool_definitions::get_tool_definitions(),
        &settings.enabled_optional_tools,
    )
    .into_iter()
    .filter(|definition| {
        definition["function"]["name"]
            .as_str()
            .is_some_and(|name| requested.contains(name))
    })
    .collect::<Vec<_>>();
    if definitions.len() != requested.len() {
        return Err("Configuration d'automatisation invalide.".into());
    }
    Ok(definitions)
}

async fn build_system(skill_ids: &[String]) -> Result<String, String> {
    if skill_ids.len() > 8 {
        return Err("Configuration d'automatisation invalide.".into());
    }
    let mut sections = Vec::with_capacity(skill_ids.len());
    for id in skill_ids {
        let skill = crate::services::agent_local::tool_skill_loader::load_skill_with_metadata(id)
            .await
            .map_err(|_| "Skill d'automatisation indisponible.".to_string())?;
        sections.push(format!(
            "<skill name=\"{}\">\n{}\n</skill>",
            escape_xml(&skill.name),
            skill.content
        ));
    }
    Ok(format!(
        "You execute one scheduled automation. Follow the user instruction exactly, use only the provided tools, and report only verified results. Stop on any missing dependency or failed required check.{}",
        if sections.is_empty() {
            String::new()
        } else {
            format!("\n\n<loaded_skills>\n{}\n</loaded_skills>", sections.join("\n\n"))
        }
    ))
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn selects_only_requested_known_tools() {
        let tools = selected_tools(&["read_file".into(), "grep".into()])
            .await
            .unwrap();
        let names = tools
            .iter()
            .filter_map(|tool| tool["function"]["name"].as_str())
            .collect::<BTreeSet<_>>();

        assert_eq!(names, BTreeSet::from(["grep", "read_file"]));
        assert!(selected_tools(&["missing".into()]).await.is_err());
    }

    #[test]
    fn escapes_agent_names_before_embedding() {
        assert_eq!(escape_xml("a<&\""), "a&lt;&amp;&quot;");
    }
}
