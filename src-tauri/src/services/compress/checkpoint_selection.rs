use std::collections::BTreeSet;

use super::checkpoint_messages::SelectedCheckpointMessage;
use super::checkpoint_units::{CheckpointUnit, CheckpointUnitKind};
use crate::services::agent_local::types_session::AgentMessage;

#[derive(Debug, Clone, Copy)]
pub struct CheckpointSelectionLimits {
    pub recent_message_count: u8,
    pub tool_tokens: u32,
    pub tool_tokens_per_result: u32,
    pub max_tool_events: u16,
    pub total_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct CheckpointSelection {
    pub units: Vec<CheckpointUnit>,
    pub messages: Vec<SelectedCheckpointMessage>,
    pub active_turn_tokens: u32,
}

pub fn select(
    source: &[AgentMessage],
    limits: CheckpointSelectionLimits,
) -> Result<CheckpointSelection, &'static str> {
    let units = super::checkpoint_units::build(source)?;
    source
        .iter()
        .try_for_each(super::checkpoint_reasoning::validate)?;
    let mut selected_starts = BTreeSet::new();
    let mut usage = Usage::default();
    select_role_quotas(&units, limits, &mut usage, &mut selected_starts);
    fill_message_slots(&units, limits, &mut usage, &mut selected_starts);

    let mut selected = Vec::new();
    let mut active_turn_tokens = 0_u32;
    for unit in units.iter().rev() {
        if unit.required {
            push_exact(source, unit, &mut selected);
            active_turn_tokens = active_turn_tokens.saturating_add(unit.estimated_tokens);
        } else if selected_starts.contains(&unit.message_indexes.start) {
            push_exact(source, unit, &mut selected);
        } else if unit.kind == CheckpointUnitKind::ToolChain {
            select_tool_chain(source, unit, limits, &mut usage, &mut selected);
        }
    }
    selected.sort_by_key(SelectedCheckpointMessage::source_index);
    Ok(CheckpointSelection {
        units,
        messages: selected,
        active_turn_tokens,
    })
}

#[derive(Default)]
struct Usage {
    users: u8,
    assistants: u8,
    messages: u8,
    tools: u32,
    tool_events: u16,
    total: u32,
}

fn select_role_quotas(
    units: &[CheckpointUnit],
    limits: CheckpointSelectionLimits,
    usage: &mut Usage,
    selected: &mut BTreeSet<usize>,
) {
    let user_quota = limits.recent_message_count.saturating_add(1) / 2;
    let assistant_quota = limits.recent_message_count / 2;
    for unit in units.iter().rev() {
        let role_has_space = match unit.kind {
            CheckpointUnitKind::UserMessage => usage.users < user_quota,
            CheckpointUnitKind::AssistantMessage => usage.assistants < assistant_quota,
            _ => false,
        };
        if role_has_space && fits_total(unit.estimated_tokens, usage.total, limits.total_tokens) {
            selected.insert(unit.message_indexes.start);
            usage.total = usage.total.saturating_add(unit.estimated_tokens);
            usage.messages = usage.messages.saturating_add(1);
            match unit.kind {
                CheckpointUnitKind::UserMessage => usage.users = usage.users.saturating_add(1),
                CheckpointUnitKind::AssistantMessage => {
                    usage.assistants = usage.assistants.saturating_add(1)
                }
                _ => {}
            }
        }
    }
}

fn fill_message_slots(
    units: &[CheckpointUnit],
    limits: CheckpointSelectionLimits,
    usage: &mut Usage,
    selected: &mut BTreeSet<usize>,
) {
    for unit in units.iter().rev() {
        if usage.messages >= limits.recent_message_count {
            break;
        }
        if !matches!(
            unit.kind,
            CheckpointUnitKind::UserMessage | CheckpointUnitKind::AssistantMessage
        ) || selected.contains(&unit.message_indexes.start)
            || !fits_total(unit.estimated_tokens, usage.total, limits.total_tokens)
        {
            continue;
        }
        selected.insert(unit.message_indexes.start);
        usage.total = usage.total.saturating_add(unit.estimated_tokens);
        usage.messages = usage.messages.saturating_add(1);
    }
}

fn select_tool_chain(
    source: &[AgentMessage],
    unit: &CheckpointUnit,
    limits: CheckpointSelectionLimits,
    usage: &mut Usage,
    output: &mut Vec<SelectedCheckpointMessage>,
) {
    let events = source[unit.message_indexes.clone()]
        .iter()
        .filter(|message| message.role == "tool")
        .count()
        .min(usize::from(u16::MAX)) as u16;
    if usage.tool_events.saturating_add(events)
        > limits
            .max_tool_events
            .min(super::checkpoint_tools::MAX_TOOL_EVENTS as u16)
    {
        return;
    }
    let transformed = transform_tool_chain(source, unit, limits.tool_tokens_per_result);
    let tokens = transformed.iter().fold(0u32, |total, selected| {
        total.saturating_add(super::token_estimate::estimate_checkpoint_message_tokens(
            selected.message(),
        ))
    });
    if tokens > limits.tool_tokens.saturating_sub(usage.tools)
        || !fits_total(tokens, usage.total, limits.total_tokens)
    {
        return;
    }
    output.extend(transformed);
    usage.tools = usage.tools.saturating_add(tokens);
    usage.tool_events = usage.tool_events.saturating_add(events);
    usage.total = usage.total.saturating_add(tokens);
}

fn transform_tool_chain(
    source: &[AgentMessage],
    unit: &CheckpointUnit,
    tokens_per_result: u32,
) -> Vec<SelectedCheckpointMessage> {
    source[unit.message_indexes.clone()]
        .iter()
        .enumerate()
        .map(|(offset, message)| {
            let source_index = unit.message_indexes.start + offset;
            if message.role != "tool" {
                return super::checkpoint_messages::exact(source_index, message);
            }
            let excerpt = super::checkpoint_tools::excerpt_result(message, tokens_per_result);
            if excerpt.content == message.content {
                super::checkpoint_messages::exact(source_index, message)
            } else {
                SelectedCheckpointMessage::ToolResultExcerpt {
                    source_index,
                    message: excerpt,
                }
            }
        })
        .collect()
}

fn fits_total(value: u32, used: u32, limit: u32) -> bool {
    value <= limit.saturating_sub(used)
}

fn push_exact(
    source: &[AgentMessage],
    unit: &CheckpointUnit,
    output: &mut Vec<SelectedCheckpointMessage>,
) {
    output.extend(
        unit.message_indexes
            .clone()
            .map(|index| super::checkpoint_messages::exact(index, &source[index])),
    );
}
