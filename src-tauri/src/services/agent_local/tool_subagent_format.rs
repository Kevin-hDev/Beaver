use super::types_session::{AgentSession, AgentSessionMeta};

pub(super) fn format_meta(meta: AgentSessionMeta) -> String {
    let activity = meta
        .subagent_last_activity
        .as_ref()
        .map(|activity| text_field(&activity.label))
        .unwrap_or_default();
    format!(
        "- id=\"{}\" name=\"{}\" type=\"{}\" status=\"{}\" run_id=\"{}\" description=\"{}\" last_activity=\"{}\"",
        xml_attr(&meta.id),
        xml_attr(&meta.name),
        xml_attr(&meta.subagent_type.unwrap_or_else(|| "explorer".to_string())),
        xml_attr(&meta.subagent_status.unwrap_or_else(|| "completed".to_string())),
        xml_attr(&meta.subagent_run_id.unwrap_or_default()),
        xml_attr(&meta.subagent_description.unwrap_or_default()),
        xml_attr(&activity)
    )
}

pub(super) fn format_child(child: &AgentSession) -> String {
    let activity = child
        .subagent_last_activity
        .as_ref()
        .map(|activity| {
            format!(
                "<last_activity kind=\"{}\" label=\"{}\">{}</last_activity>",
                xml_attr(&activity.kind),
                xml_attr(&activity.label),
                xml_text(activity.detail.as_deref().unwrap_or(""))
            )
        })
        .unwrap_or_else(|| "<last_activity />".to_string());
    format!(
        "<subagent id=\"{}\" name=\"{}\" type=\"{}\" status=\"{}\" run_id=\"{}\" queued_prompts=\"{}\">\n<description>{}</description>\n{}\n<summary>{}</summary>\n</subagent>",
        xml_attr(&child.id),
        xml_attr(&child.name),
        xml_attr(child.subagent_type.as_deref().unwrap_or("explorer")),
        xml_attr(child.subagent_status.as_deref().unwrap_or("completed")),
        xml_attr(child.subagent_run_id.as_deref().unwrap_or("")),
        child.subagent_queued_prompts.len(),
        xml_text(child.subagent_description.as_deref().unwrap_or("")),
        activity,
        xml_text(child.subagent_summary.as_deref().unwrap_or(""))
    )
}

fn text_field(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn xml_attr(value: &str) -> String {
    text_field(value)
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::types_session::{AgentSessionMeta, SubagentLastActivity};
    use chrono::Utc;

    #[test]
    fn format_meta_escapes_fields() {
        let meta = AgentSessionMeta {
            id: "child<&".into(),
            name: "Gemini\"tor".into(),
            created_at: Utc::now(),
            updated_at: Some(Utc::now()),
            archived_at: None,
            model: "llama3".into(),
            provider: "ollama".into(),
            thinking_enabled: false,
            reasoning_mode: None,
            message_count: 0,
            is_heartbeat: false,
            is_gateway: false,
            gateway_channel_key: None,
            project_id: None,
            parent_session_id: Some("parent".into()),
            subagent_type: Some("explorer".into()),
            subagent_status: Some("running".into()),
            subagent_run_id: Some("run\"1".into()),
            subagent_description: Some("<analyse>".into()),
            subagent_color_key: Some("geminitor".into()),
            subagent_summary: None,
            subagent_last_activity: Some(SubagentLastActivity {
                kind: "tool".into(),
                label: "bash <ok>".into(),
                detail: None,
                updated_at: Utc::now(),
            }),
            clone_parent_session_id: None,
            clone_parent_message_id: None,
            clone_mode: None,
            clone_root_session_id: None,
            git_branch: None,
        };

        let line = format_meta(meta);

        assert!(line.contains("id=\"child&lt;&amp;\""));
        assert!(line.contains("name=\"Gemini&quot;tor\""));
        assert!(line.contains("run_id=\"run&quot;1\""));
        assert!(line.contains("description=\"&lt;analyse&gt;\""));
        assert!(line.contains("last_activity=\"bash &lt;ok&gt;\""));
    }
}
