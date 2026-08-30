#![allow(
    dead_code,
    reason = "the shared compression orchestrator consumes this staged document in Task 11"
)]

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::checkpoint_messages::SelectedCheckpointMessage;
use super::compression_redaction::redact_checkpoint_text;
use super::profile_types::CompressionTrigger;
use crate::services::agent_local::types_message::{AgentMessage, AgentMessageKind};

const CHECKPOINT_FORMAT_VERSION: u16 = 1;
const MAX_SECTIONS: usize = 32;
const MAX_SECTION_NAME_CHARS: usize = 64;

#[derive(Debug, Clone)]
pub struct CheckpointSection {
    pub name: String,
    pub content: String,
}

#[derive(Serialize)]
struct CheckpointBody<'a> {
    format_version: u16,
    checkpoint_id: &'a str,
    summary: Option<&'a str>,
    sections: &'a BTreeMap<String, String>,
}

pub fn assemble(
    selected: &[SelectedCheckpointMessage],
    summary: Option<&str>,
    sections: &[CheckpointSection],
    trigger: CompressionTrigger,
) -> Result<Vec<AgentMessage>, &'static str> {
    let (mut completed, active) = retained_turns(selected);
    let checkpoint = checkpoint_turn(summary, sections, trigger)?;
    completed.extend(checkpoint);
    completed.extend(active);
    crate::services::agent_local::conversation_history_validation::validate(&completed)
        .map_err(|_| "compression_candidate_invalid")?;
    Ok(completed)
}

fn retained_turns(
    selected: &[SelectedCheckpointMessage],
) -> (Vec<AgentMessage>, Vec<AgentMessage>) {
    let mut by_turn = BTreeMap::<String, Vec<AgentMessage>>::new();
    let mut order = Vec::new();
    for item in selected {
        let message = item.message().clone();
        if message.message_kind.is_some() {
            continue;
        }
        if !by_turn.contains_key(&message.turn_id) {
            order.push(message.turn_id.clone());
        }
        by_turn
            .entry(message.turn_id.clone())
            .or_default()
            .push(message);
    }
    let mut completed = Vec::new();
    let mut active = Vec::new();
    for turn_id in order {
        let Some(turn) = by_turn.remove(&turn_id) else {
            continue;
        };
        if turn
            .first()
            .is_some_and(|message| message.role == "user" && message.content.trim() == "/compress")
        {
            continue;
        }
        if valid_terminal_turn(&turn) {
            completed.extend(turn);
        } else if valid_active_turn(&turn) {
            active = turn;
        }
    }
    (completed, active)
}

fn valid_terminal_turn(turn: &[AgentMessage]) -> bool {
    turn.first().is_some_and(|message| message.role == "user")
        && crate::services::agent_local::conversation_compaction::is_terminal_turn(turn)
        && crate::services::agent_local::conversation_history_validation::validate(turn).is_ok()
}

fn valid_active_turn(turn: &[AgentMessage]) -> bool {
    turn.first().is_some_and(|message| message.role == "user")
        && !crate::services::agent_local::conversation_compaction::is_terminal_turn(turn)
        && crate::services::agent_local::conversation_history_validation::validate(turn).is_ok()
}

fn checkpoint_turn(
    summary: Option<&str>,
    sections: &[CheckpointSection],
    _trigger: CompressionTrigger,
) -> Result<Vec<AgentMessage>, &'static str> {
    let checkpoint_id = uuid::Uuid::new_v4().to_string();
    let sections = sanitized_sections(sections)?;
    let summary = summary.map(redact_checkpoint_text);
    let body = CheckpointBody {
        format_version: CHECKPOINT_FORMAT_VERSION,
        checkpoint_id: &checkpoint_id,
        summary: summary.as_deref(),
        sections: &sections,
    };
    let content =
        serde_json::to_string_pretty(&body).map_err(|_| "compression_candidate_invalid")?;
    let turn_id = AgentMessage::new_turn_id();
    let user = technical_message(
        "user",
        content,
        turn_id.clone(),
        AgentMessageKind::CompressionCheckpoint,
    );
    let assistant = technical_message(
        "assistant",
        super::engine::BOUNDARY_CONTENT.to_string(),
        turn_id,
        AgentMessageKind::CompressionBoundary,
    );
    Ok(vec![user, assistant])
}

fn sanitized_sections(
    sections: &[CheckpointSection],
) -> Result<BTreeMap<String, String>, &'static str> {
    if sections.len() > MAX_SECTIONS {
        return Err("compression_candidate_invalid");
    }
    let mut names = BTreeSet::new();
    let mut output = BTreeMap::new();
    for section in sections {
        let name = section.name.trim();
        if name.is_empty()
            || name.chars().count() > MAX_SECTION_NAME_CHARS
            || name.chars().any(char::is_control)
            || !names.insert(name.to_string())
        {
            return Err("compression_candidate_invalid");
        }
        output.insert(name.to_string(), redact_checkpoint_text(&section.content));
    }
    Ok(output)
}

fn technical_message(
    role: &str,
    content: String,
    turn_id: String,
    kind: AgentMessageKind,
) -> AgentMessage {
    AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        turn_id,
        role: role.to_string(),
        content,
        message_kind: Some(kind),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        continuation: None,
        replay_source: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: chrono::Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        skill_ids: None,
        stream_run_id: None,
        stream_part: None,
    }
}
