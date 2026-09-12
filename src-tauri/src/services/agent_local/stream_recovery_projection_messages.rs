use super::stream_recovery_projection::EventProjection;
use super::stream_recovery_record::{RecoverableToolResult, StreamRecoveryHeader};
use super::types_message::{
    AgentMessage, PersistedToolResultMeta, SavedSegment, ToolActivityRecord, ToolCallRequest,
};

pub(super) fn finish(
    mut state: EventProjection,
    header: &StreamRecoveryHeader,
) -> Result<Vec<AgentMessage>, String> {
    if !state.content.is_empty() || !state.thinking.is_empty() || !state.activities.is_empty() {
        if state.segments.len() >= super::session_limits::MAX_MESSAGES_PER_SESSION {
            return Err("stream_recovery_invalid".into());
        }
        state.segments.push(SavedSegment {
            thinking: (!state.thinking.is_empty()).then_some(state.thinking),
            tools: state.activities,
            content: state.content,
            phase: state.phase,
        });
    }
    let content = state
        .segments
        .iter()
        .map(|segment| segment.content.as_str())
        .collect::<String>();
    let thinking = state
        .segments
        .iter()
        .filter_map(|segment| segment.thinking.as_deref())
        .collect::<String>();
    let mut messages = Vec::new();
    if !content.is_empty() || !thinking.is_empty() || !state.calls.is_empty() {
        messages.push(base_message(
            header.assistant_message_id.clone(),
            header,
            "assistant",
            content,
            (!thinking.is_empty()).then_some(thinking),
            (!state.calls.is_empty()).then_some(state.calls),
            None,
            None,
            (!state.segments.is_empty()).then_some(state.segments),
            None,
        ));
    }
    messages.extend(tool_messages(header, state.results)?);
    Ok(messages)
}

fn tool_messages(
    header: &StreamRecoveryHeader,
    mut results: Vec<RecoverableToolResult>,
) -> Result<Vec<AgentMessage>, String> {
    results.sort_by_key(|result| result.tool_call_index);
    let mut chats = Vec::new();
    let mut outcome = super::tool_execution_outcome::ToolExecutionOutcome::default();
    for result in &results {
        let follow_up = super::tool_executor_results::push_tool_message(
            &mut chats,
            &result.name,
            super::types_tools::ToolResult::from_recovery(result),
            result.tool_call_id.as_deref(),
        );
        outcome.record(follow_up);
    }
    outcome.apply_follow_ups(&mut chats)?;
    Ok(chats
        .into_iter()
        .zip(results)
        .map(|(chat, result)| {
            let activities = (!result.artifacts.is_empty()).then(|| {
                vec![ToolActivityRecord::artifact_carrier(
                    result.name.clone(),
                    result.artifacts,
                )]
            });
            base_message(
                result.message_id,
                header,
                "tool",
                chat.content,
                None,
                None,
                Some(result.name),
                chat.tool_call_id,
                None,
                activities,
            )
        })
        .collect())
}

#[allow(clippy::too_many_arguments)]
fn base_message(
    id: String,
    header: &StreamRecoveryHeader,
    role: &str,
    content: String,
    thinking: Option<String>,
    tool_calls: Option<Vec<ToolCallRequest>>,
    tool_name: Option<String>,
    tool_call_id: Option<String>,
    segments: Option<Vec<SavedSegment>>,
    tool_activities: Option<Vec<ToolActivityRecord>>,
) -> AgentMessage {
    AgentMessage {
        id,
        turn_id: header.turn_id.clone(),
        role: role.into(),
        content,
        message_kind: None,
        thinking,
        tool_calls,
        tool_name,
        tool_call_id,
        continuation: None,
        replay_source: None,
        tool_activities,
        segments,
        files: Vec::new(),
        timestamp: header.created_at,
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        skill_ids: None,
        stream_run_id: Some(header.request_id.clone()),
        stream_part: Some("checkpoint".into()),
    }
}

pub(super) fn activity_for_call(
    name: &str,
    args: serde_json::Value,
    domain: Option<String>,
) -> ToolActivityRecord {
    ToolActivityRecord {
        name: name.into(),
        summary: String::new(),
        domain,
        resolved_path: None,
        args: Some(args),
        result: None,
        is_error: None,
        result_meta: None,
        content: None,
        old_text: None,
        new_text: None,
        start_line: None,
        affected_paths: Vec::new(),
        file_changes: Vec::new(),
        artifacts: Vec::new(),
    }
}

pub(super) fn apply_result(activity: &mut ToolActivityRecord, result: &RecoverableToolResult) {
    activity.result = Some(result.content.clone());
    activity.is_error = Some(result.is_error);
    activity.result_meta = Some(PersistedToolResultMeta {
        status: result.status,
        error: result.error.clone(),
        warnings: result.warnings.clone(),
        truncated: result.truncated,
    });
    activity.resolved_path.clone_from(&result.resolved_path);
    activity.domain.clone_from(&result.domain);
    activity.affected_paths.clone_from(&result.affected_paths);
    activity.file_changes.clone_from(&result.file_changes);
    activity.start_line = result.start_line.and_then(|value| u32::try_from(value).ok());
    activity.artifacts.clone_from(&result.artifacts);
}
