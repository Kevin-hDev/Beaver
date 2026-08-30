#![allow(
    dead_code,
    reason = "the compression orchestrator consumes the staged selection in Task 10"
)]

use super::checkpoint_messages::SelectedCheckpointMessage;
use super::checkpoint_units::{CheckpointUnit, CheckpointUnitKind};
use crate::services::agent_local::types_session::AgentMessage;

#[derive(Debug, Clone, Copy)]
pub struct CheckpointSelectionLimits {
    pub user_tokens: u32,
    pub assistant_tokens: u32,
    pub tool_tokens: u32,
    pub tool_tokens_per_result: u32,
    pub max_tool_events: u16,
    pub total_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct CheckpointSelection {
    pub units: Vec<CheckpointUnit>,
    pub messages: Vec<SelectedCheckpointMessage>,
    pub estimated_tokens: u32,
}

pub fn select(
    source: &[AgentMessage],
    limits: CheckpointSelectionLimits,
) -> Result<CheckpointSelection, &'static str> {
    let units = super::checkpoint_units::build(source)?;
    validate_reasoning(source)?;
    let mut selected = Vec::new();
    let mut used = Usage::default();
    for unit in units.iter().rev() {
        if unit.required {
            if unit.estimated_tokens > limits.total_tokens.saturating_sub(used.total) {
                return Err(crate::services::agent_local::context_capacity_error::CODE);
            }
            push_exact(source, unit, &mut selected);
            used.total = used.total.saturating_add(unit.estimated_tokens);
            continue;
        }
        match unit.kind {
            CheckpointUnitKind::UserMessage => {
                select_user(source, unit, limits, &mut used, &mut selected)
            }
            CheckpointUnitKind::AssistantMessage => {
                if fits(
                    unit.estimated_tokens,
                    used.assistant,
                    limits.assistant_tokens,
                    used.total,
                    limits.total_tokens,
                ) {
                    push_exact(source, unit, &mut selected);
                    used.assistant = used.assistant.saturating_add(unit.estimated_tokens);
                    used.total = used.total.saturating_add(unit.estimated_tokens);
                }
            }
            CheckpointUnitKind::ToolChain => {
                select_tool_chain(source, unit, limits, &mut used, &mut selected)
            }
            CheckpointUnitKind::ActiveTurn => unreachable!("required units handled above"),
        }
    }
    selected.sort_by_key(SelectedCheckpointMessage::source_index);
    Ok(CheckpointSelection {
        units,
        messages: selected,
        estimated_tokens: used.total,
    })
}

#[derive(Default)]
struct Usage {
    user: u32,
    assistant: u32,
    tools: u32,
    tool_events: u16,
    total: u32,
}

fn select_user(
    source: &[AgentMessage],
    unit: &CheckpointUnit,
    limits: CheckpointSelectionLimits,
    used: &mut Usage,
    out: &mut Vec<SelectedCheckpointMessage>,
) {
    let remaining = limits
        .user_tokens
        .saturating_sub(used.user)
        .min(limits.total_tokens.saturating_sub(used.total));
    if remaining == 0 {
        return;
    }
    if unit.estimated_tokens <= remaining {
        push_exact(source, unit, out);
        used.user = used.user.saturating_add(unit.estimated_tokens);
        used.total = used.total.saturating_add(unit.estimated_tokens);
    } else {
        let selected = super::checkpoint_messages::truncate_user(
            unit.message_indexes.start,
            &source[unit.message_indexes.start],
            remaining,
        );
        let tokens = super::token_estimate::estimate_checkpoint_message_tokens(selected.message());
        out.push(selected);
        used.user = used.user.saturating_add(tokens);
        used.total = used.total.saturating_add(tokens);
    }
}

fn select_tool_chain(
    source: &[AgentMessage],
    unit: &CheckpointUnit,
    limits: CheckpointSelectionLimits,
    used: &mut Usage,
    out: &mut Vec<SelectedCheckpointMessage>,
) {
    let events = source[unit.message_indexes.clone()]
        .iter()
        .filter(|message| message.role == "tool")
        .count() as u16;
    if used.tool_events.saturating_add(events)
        > limits
            .max_tool_events
            .min(super::checkpoint_tools::MAX_TOOL_EVENTS as u16)
    {
        return;
    }
    let transformed = source[unit.message_indexes.clone()]
        .iter()
        .enumerate()
        .map(|(offset, message)| {
            let source_index = unit.message_indexes.start + offset;
            if message.role == "tool" {
                let excerpt =
                    super::checkpoint_tools::excerpt_result(message, limits.tool_tokens_per_result);
                if excerpt.content == message.content {
                    super::checkpoint_messages::exact(source_index, message)
                } else {
                    SelectedCheckpointMessage::ToolResultExcerpt {
                        source_index,
                        message: excerpt,
                    }
                }
            } else {
                super::checkpoint_messages::exact(source_index, message)
            }
        })
        .collect::<Vec<_>>();
    let tokens = transformed.iter().fold(0u32, |total, selected| {
        total.saturating_add(super::token_estimate::estimate_checkpoint_message_tokens(
            selected.message(),
        ))
    });
    if !fits(
        tokens,
        used.tools,
        limits.tool_tokens,
        used.total,
        limits.total_tokens,
    ) {
        return;
    }
    out.extend(transformed);
    used.tools = used.tools.saturating_add(tokens);
    used.tool_events = used.tool_events.saturating_add(events);
    used.total = used.total.saturating_add(tokens);
}

fn validate_reasoning(source: &[AgentMessage]) -> Result<(), &'static str> {
    source
        .iter()
        .try_for_each(super::checkpoint_reasoning::validate)
}

fn fits(
    value: u32,
    category_used: u32,
    category_limit: u32,
    total_used: u32,
    total_limit: u32,
) -> bool {
    value <= category_limit.saturating_sub(category_used)
        && value <= total_limit.saturating_sub(total_used)
}

fn push_exact(
    source: &[AgentMessage],
    unit: &CheckpointUnit,
    out: &mut Vec<SelectedCheckpointMessage>,
) {
    out.extend(
        unit.message_indexes
            .clone()
            .map(|index| super::checkpoint_messages::exact(index, &source[index])),
    );
}
