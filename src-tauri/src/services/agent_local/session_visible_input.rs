use crate::models::agent_session_contract::{
    FileAttachmentView, SavedSegmentView, ToolActivityRecordView, ToolFileChangeView,
    VisibleMessageInput,
};

use super::types_message::{AgentMessage, FileAttachment, SavedSegment, ToolActivityRecord};

const MAX_VISIBLE_MESSAGE_BYTES: usize = 2 * 1024 * 1024;
const MAX_MESSAGE_TEXT_BYTES: usize = 1024 * 1024;
const MAX_FILES: usize = 32;
const MAX_SKILLS: usize = 8;
pub(super) const MAX_VISIBLE_MESSAGE_BATCH: usize = 16;

pub(super) fn into_message(input: VisibleMessageInput, turn_id: String) -> Result<AgentMessage, String> {
    validate_input(&input)?;
    let message = AgentMessage {
        id: input.id, turn_id, role: input.role, content: input.content,
        thinking: input.thinking,
        tool_calls: input.tool_calls.map(|calls| calls.into_iter().map(|call| {
            super::types_message::ToolCallRequest {
                id: call.id, extra_content: None,
                function: super::types_message::ToolCallRequestFunction {
                    name: call.function.name, arguments: call.function.arguments,
                },
            }
        }).collect()),
        tool_name: input.tool_name, tool_call_id: input.tool_call_id, continuation: None,
        tool_activities: input.tool_activities.map(|items| {
            items.into_iter().map(activity_input).collect::<Result<Vec<_>, _>>()
        }).transpose()?,
        segments: input.segments.map(|items| {
            items.into_iter().map(segment_input).collect::<Result<Vec<_>, _>>()
        }).transpose()?,
        files: input.files.into_iter().map(file_input).collect(),
        timestamp: input.timestamp, tokens: input.tokens,
        work_duration_ms: input.work_duration_ms, skill_names: input.skill_names,
        stream_run_id: input.stream_run_id, stream_part: input.stream_part,
    };
    message.validate_stream_metadata()?;
    Ok(message)
}

fn validate_input(input: &VisibleMessageInput) -> Result<(), String> {
    let invalid = || "Message invalide".to_string();
    super::session_migration_ids::validate_id(&input.id).map_err(|_| invalid())?;
    if !matches!(input.role.as_str(), "user" | "assistant" | "tool")
        || invalid_text(&input.content)
        || input.thinking.as_deref().is_some_and(invalid_text)
        || input.files.len() > MAX_FILES
        || input.skill_names.as_ref().is_some_and(|items| items.len() > MAX_SKILLS)
        || input.tool_calls.as_ref().is_some_and(|items| {
            items.len() > crate::services::reasoning_continuity::limits::MAX_TOOL_CALLS
        })
        || input.segments.as_ref().is_some_and(|items| {
            items.len() > crate::services::reasoning_continuity::limits::MAX_NATIVE_ITEMS
        })
        || visible_activities(input).count()
            > crate::services::reasoning_continuity::limits::MAX_NATIVE_ITEMS
        || serde_json::to_vec(input).map_err(|_| invalid())?.len() > MAX_VISIBLE_MESSAGE_BYTES
    {
        return Err(invalid());
    }
    for call in input.tool_calls.iter().flatten() {
        crate::services::reasoning_continuity::limits::validate_provider_call_id(&call.id)
            .map_err(|_| invalid())?;
        crate::services::reasoning_continuity::limits::validate_tool_name(&call.function.name)
            .map_err(|_| invalid())?;
        crate::services::reasoning_continuity::limits::validate_json_depth(&call.function.arguments)
            .map_err(|_| invalid())?;
    }
    for activity in visible_activities(input) {
        if let Some(args) = activity.args.as_ref() {
            crate::services::reasoning_continuity::limits::validate_json_depth(args)
                .map_err(|_| invalid())?;
        }
    }
    Ok(())
}

fn invalid_text(value: &str) -> bool {
    value.len() > MAX_MESSAGE_TEXT_BYTES || value.contains('\0')
}

fn visible_activities(input: &VisibleMessageInput) -> impl Iterator<Item = &ToolActivityRecordView> {
    input.tool_activities.iter().flatten().chain(
        input.segments.iter().flatten().flat_map(|segment| &segment.tools),
    )
}

fn file_input(file: FileAttachmentView) -> FileAttachment {
    FileAttachment { name: file.name, path: file.path, mime_type: file.mime_type,
        size: file.size, thumbnail: file.thumbnail, access_grant: file.access_grant }
}

fn segment_input(segment: SavedSegmentView) -> Result<SavedSegment, String> {
    Ok(SavedSegment { thinking: segment.thinking,
        tools: segment.tools.into_iter().map(activity_input).collect::<Result<Vec<_>, _>>()?,
        content: segment.content, phase: match segment.phase.as_deref() {
            Some("work") => Some(super::types_stream::TokenPhase::Work),
            Some("final") => Some(super::types_stream::TokenPhase::Final),
            None => None, _ => return Err("Message invalide".to_string()),
        } })
}

fn activity_input(record: ToolActivityRecordView) -> Result<ToolActivityRecord, String> {
    Ok(ToolActivityRecord { name: record.name, summary: record.summary, domain: record.domain,
        resolved_path: record.resolved_path, args: record.args, result: record.result,
        is_error: record.is_error,
        result_meta: record.result_meta.map(serde_json::from_value).transpose()
            .map_err(|_| "Message invalide".to_string())?, content: record.content,
        old_text: record.old_text, new_text: record.new_text, start_line: record.start_line,
        affected_paths: record.affected_paths,
        file_changes: record.file_changes.into_iter().map(file_change_input).collect::<Result<Vec<_>, _>>()? })
}

fn file_change_input(change: ToolFileChangeView) -> Result<super::types_tools::ToolFileChange, String> {
    Ok(super::types_tools::ToolFileChange { path: change.path, status: match change.status.as_str() {
        "added" => super::types_tools::ToolFileChangeStatus::Added,
        "modified" => super::types_tools::ToolFileChangeStatus::Modified,
        "deleted" => super::types_tools::ToolFileChangeStatus::Deleted,
        _ => return Err("Message invalide".to_string()),
    }, additions: change.additions, deletions: change.deletions,
        diff: change.diff.map(serde_json::from_value).transpose()
            .map_err(|_| "Message invalide".to_string())? })
}
