use super::types_ollama::ChatMessage;

const MAX_HISTORY_TOOL_ID_CHARS: usize = 512;
const MAX_HISTORY_TOOL_NAME_CHARS: usize = 128;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HistoryRepairReport {
    pub repaired_tool_chains: usize,
    pub dropped_tool_results: usize,
}

pub struct HistoryUnit<'a> {
    pub messages: Vec<&'a ChatMessage>,
    pub is_tool_chain: bool,
    pub valid: bool,
}

pub fn repair_invalid_history(messages: &mut Vec<ChatMessage>) -> HistoryRepairReport {
    if history_is_valid(messages) {
        return HistoryRepairReport::default();
    }
    let mut report = HistoryRepairReport::default();
    let mut repaired = Vec::with_capacity(messages.len());
    let mut input = std::mem::take(messages).into_iter().peekable();
    while let Some(message) = input.next() {
        let expected = expected_results(&message);
        if message.role == "assistant" && expected > 0 {
            let mut chain = vec![message];
            while chain.len() <= expected
                && input.peek().is_some_and(|next| next.role == "tool")
            {
                let Some(result) = input.next() else {
                    break;
                };
                chain.push(result);
            }
            let calls = chain[0].tool_calls.as_deref().unwrap_or_default();
            if chain.len() == expected + 1 && matching_tool_results(calls, &chain[1..]) {
                repaired.extend(chain);
                continue;
            }
            report.repaired_tool_chains = report.repaired_tool_chains.saturating_add(1);
            report.dropped_tool_results = report
                .dropped_tool_results
                .saturating_add(chain.len().saturating_sub(1));
            let mut assistant = chain.remove(0);
            assistant.tool_calls = None;
            if !assistant.content.is_empty() || assistant.display_thinking.is_some() {
                repaired.push(assistant);
            }
        } else if message.role == "tool" {
            report.dropped_tool_results = report.dropped_tool_results.saturating_add(1);
        } else {
            repaired.push(message);
        }
    }
    *messages = repaired;
    report
}

fn history_is_valid(messages: &[ChatMessage]) -> bool {
    let mut index = 0usize;
    while let Some(message) = messages.get(index) {
        let expected = expected_results(message);
        if message.role == "assistant" && expected > 0 {
            let end = index.saturating_add(expected).saturating_add(1);
            let start = index.saturating_add(1);
            let Some(results) = messages.get(start..end) else {
                return false;
            };
            let calls = message.tool_calls.as_deref().unwrap_or_default();
            if !matching_tool_results(calls, results) {
                return false;
            }
            index = end;
        } else if message.role == "tool" {
            return false;
        } else {
            index += 1;
        }
    }
    true
}

pub fn atomic_units(messages: Vec<&ChatMessage>) -> Vec<HistoryUnit<'_>> {
    let mut units = Vec::new();
    let mut messages = messages.into_iter().peekable();
    while let Some(message) = messages.next() {
        let expected = expected_results(message);
        if message.role == "assistant" && expected > 0 {
            let mut chain = vec![message];
            while chain.len() <= expected
                && messages.peek().is_some_and(|next| next.role == "tool")
            {
                let Some(result) = messages.next() else {
                    break;
                };
                chain.push(result);
            }
            let calls = message.tool_calls.as_deref().unwrap_or_default();
            let valid = chain.len() == expected + 1
                && matching_tool_result_refs(calls, &chain[1..]);
            units.push(HistoryUnit {
                messages: chain,
                is_tool_chain: true,
                valid,
            });
        } else {
            units.push(HistoryUnit {
                messages: vec![message],
                is_tool_chain: message.role == "tool",
                valid: message.role != "tool",
            });
        }
    }
    units
}

fn expected_results(message: &ChatMessage) -> usize {
    message
        .tool_calls
        .as_ref()
        .filter(|calls| !calls.is_empty())
        .map_or(0, Vec::len)
}

fn matching_tool_results(
    calls: &[super::types_ollama::ToolCallOllama],
    results: &[ChatMessage],
) -> bool {
    call_ids_are_unambiguous(calls)
        && calls
        .iter()
        .zip(results)
        .all(|(call, result)| tool_result_matches(call, result))
}

fn matching_tool_result_refs(
    calls: &[super::types_ollama::ToolCallOllama],
    results: &[&ChatMessage],
) -> bool {
    call_ids_are_unambiguous(calls)
        && calls
        .iter()
        .zip(results)
        .all(|(call, result)| tool_result_matches(call, result))
}

fn call_ids_are_unambiguous(calls: &[super::types_ollama::ToolCallOllama]) -> bool {
    let with_ids = calls
        .iter()
        .filter(|call| normalized_id(call.id.as_ref()).is_some())
        .count();
    (with_ids == 0 || with_ids == calls.len())
        && calls.iter().all(|call| {
            !call.function.name.is_empty()
                && call.function.name.chars().count() <= MAX_HISTORY_TOOL_NAME_CHARS
                && call.id.as_ref().is_none_or(|id| {
                    id.chars().count() <= MAX_HISTORY_TOOL_ID_CHARS
                })
        })
        && calls.iter().enumerate().all(|(index, call)| {
            normalized_id(call.id.as_ref()).is_none_or(|id| {
                calls[..index].iter().all(|previous| {
                    normalized_id(previous.id.as_ref()) != Some(id)
                })
            })
        })
}

fn tool_result_matches(
    call: &super::types_ollama::ToolCallOllama,
    result: &ChatMessage,
) -> bool {
    let id_matches = match (
        normalized_id(call.id.as_ref()),
        normalized_id(result.tool_call_id.as_ref()),
    ) {
        (Some(expected), Some(actual)) => expected == actual,
        (None, None) => true,
        _ => false,
    };
    let name_matches = result
        .tool_name
        .as_deref()
        .is_none_or(|name| name == call.function.name);
    result.role == "tool" && id_matches && name_matches
}

fn normalized_id(value: Option<&String>) -> Option<&str> {
    value.map(String::as_str).filter(|id| !id.is_empty())
}

#[cfg(test)]
#[path = "context_budget_history_tests.rs"]
mod tests;
