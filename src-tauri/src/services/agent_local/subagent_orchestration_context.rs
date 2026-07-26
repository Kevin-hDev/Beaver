use super::types_ollama::ChatMessage;

pub const SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX: &str = "Subagent orchestration context:";
const LEGACY_GATE_MARKER: &str = "\n<subagent_final_gate final_answer_allowed=\"false\">";
const LEGACY_GATE_INSTRUCTION: &str =
    "<instruction>Final answer is locked because current-turn subagents are still running.";

pub fn replace_gate_context(
    messages: &mut Vec<ChatMessage>,
    active_count: usize,
    reports_injected: bool,
) {
    remove_gate_context(messages);
    if active_count == 0 {
        return;
    }
    let context = ChatMessage {
        role: "system".to_string(),
        content: build_gate_content(active_count, reports_injected),
        ..Default::default()
    };
    super::subagent_report_context::insert_leading_system_message(messages, context);
}

pub fn remove_gate_context(messages: &mut Vec<ChatMessage>) {
    messages.retain(|message| !is_runtime_context(message) && !is_legacy_user_gate(message));
}

pub fn build_gate_content(active_count: usize, reports_injected: bool) -> String {
    let report_state = if reports_injected {
        "Terminal reports are available in this request."
    } else {
        "No terminal report is available in this request."
    };
    format!(
        "{SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX}\n\
         <subagent_runtime_context>\n\
         <guidance>Some subagents are still working. Continue useful independent work without \
         duplicating delegated work. Terminal reports arrive automatically. If a terminal report \
         is available while other subagents are still working, use it for useful independent work \
         and defer the overall summary until every active subagent has reported. When no useful \
         independent work remains, give at most one short progress update and finish this turn \
         without a tool call.</guidance>\n\
         <report_state>{report_state}</report_state>\n\
         <active_count>{active_count}</active_count>\n\
         </subagent_runtime_context>",
    )
}

fn is_runtime_context(message: &ChatMessage) -> bool {
    message.role == "system"
        && message
            .content
            .trim_start()
            .starts_with(SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX)
}

fn is_legacy_user_gate(message: &ChatMessage) -> bool {
    message.role == "user"
        && message
            .content
            .strip_prefix(SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX)
            .is_some_and(|body| {
                body.starts_with(LEGACY_GATE_MARKER)
                    && body.contains(LEGACY_GATE_INSTRUCTION)
                    && body.ends_with("</subagent_final_gate>")
            })
}

#[cfg(test)]
#[path = "subagent_orchestration_context_tests.rs"]
mod tests;
