use crate::models::agent_session_contract::{
    AgentMessageView, FileAttachmentView, SavedSegmentView, ToolActivityRecordView,
    ToolCallFunctionView, ToolCallRequestView, ToolFileChangeView,
};

use super::types_message::{AgentMessage, FileAttachment, SavedSegment, ToolActivityRecord};

pub(crate) fn from_message(message: &AgentMessage) -> Result<AgentMessageView, String> {
    Ok(AgentMessageView {
        id: message.id.clone(),
        turn_id: message.turn_id.clone(),
        role: message.role.clone(),
        content: message.content.clone(),
        thinking: message.thinking.clone(),
        tool_calls: message.tool_calls.as_ref().map(|calls| {
            calls
                .iter()
                .map(|call| ToolCallRequestView {
                    id: call.id.clone(),
                    function: ToolCallFunctionView {
                        name: call.function.name.clone(),
                        arguments: call.function.arguments.clone(),
                    },
                })
                .collect()
        }),
        tool_name: message.tool_name.clone(),
        tool_call_id: message.tool_call_id.clone(),
        tool_activities: message
            .tool_activities
            .as_ref()
            .map(|items| items.iter().map(activity_view).collect::<Result<Vec<_>, _>>())
            .transpose()?,
        segments: message
            .segments
            .as_ref()
            .map(|items| items.iter().map(segment_view).collect::<Result<Vec<_>, _>>())
            .transpose()?,
        files: message.files.iter().map(file_view).collect(),
        timestamp: message.timestamp,
        tokens: message.tokens,
        work_duration_ms: message.work_duration_ms,
        skill_names: message.skill_names.clone(),
        stream_run_id: message.stream_run_id.clone(),
        stream_part: message.stream_part.clone(),
        reasoning_replay_status: super::session_view::replay_status(message.continuation.as_ref()),
    })
}

fn file_view(file: &FileAttachment) -> FileAttachmentView {
    FileAttachmentView {
        name: file.name.clone(), path: file.path.clone(), mime_type: file.mime_type.clone(),
        size: file.size, thumbnail: file.thumbnail.clone(), access_grant: file.access_grant.clone(),
    }
}

fn segment_view(segment: &SavedSegment) -> Result<SavedSegmentView, String> {
    Ok(SavedSegmentView {
        thinking: segment.thinking.clone(),
        tools: segment.tools.iter().map(activity_view).collect::<Result<Vec<_>, _>>()?,
        content: segment.content.clone(),
        phase: segment.phase.as_ref().map(|phase| match phase {
            super::types_stream::TokenPhase::Work => "work".to_string(),
            super::types_stream::TokenPhase::Final => "final".to_string(),
        }),
    })
}

fn activity_view(record: &ToolActivityRecord) -> Result<ToolActivityRecordView, String> {
    Ok(ToolActivityRecordView {
        name: record.name.clone(), summary: record.summary.clone(), domain: record.domain.clone(),
        resolved_path: record.resolved_path.clone(), args: record.args.clone(),
        result: record.result.clone(), is_error: record.is_error,
        result_meta: record.result_meta.as_ref().map(serde_json::to_value).transpose()
            .map_err(|_| "Session indisponible".to_string())?,
        content: record.content.clone(), old_text: record.old_text.clone(),
        new_text: record.new_text.clone(), start_line: record.start_line,
        affected_paths: record.affected_paths.clone(),
        file_changes: record.file_changes.iter().map(file_change_view).collect::<Result<Vec<_>, _>>()?,
    })
}

fn file_change_view(change: &super::types_tools::ToolFileChange) -> Result<ToolFileChangeView, String> {
    Ok(ToolFileChangeView {
        path: change.path.clone(),
        status: match change.status {
            super::types_tools::ToolFileChangeStatus::Added => "added",
            super::types_tools::ToolFileChangeStatus::Modified => "modified",
            super::types_tools::ToolFileChangeStatus::Deleted => "deleted",
        }.to_string(),
        additions: change.additions, deletions: change.deletions,
        diff: change.diff.as_ref().map(serde_json::to_value).transpose()
            .map_err(|_| "Session indisponible".to_string())?,
    })
}
