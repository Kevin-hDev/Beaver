use super::types_message::{AgentMessage, ToolActivityRecord};
use super::types_ollama::ChatMessage;

pub const CLONE_SUMMARY_PREFIX: &str = "This cloned session includes hidden branch context:";
const MAX_SUMMARY_INPUT_CHARS: usize = 120_000;
const MAX_TOOL_RESULT_CHARS: usize = 2_000;
const MAX_TRACKED_FILES: usize = 200;
const TRUNCATED_MARKER: &str = "\n[Truncated]";

pub fn hidden_context_content(summary: &str) -> String {
    format!("{CLONE_SUMMARY_PREFIX}\n\n{}", summary.trim())
}

pub fn build_summary_messages(serialized: &str, focus: Option<&str>) -> Vec<ChatMessage> {
    super::clone_summary_prompt::build_summary_messages(serialized, focus)
}

pub fn serialize_messages(messages: &[AgentMessage]) -> String {
    let mut inherited = String::new();
    let mut out = String::new();
    for msg in messages {
        // Le contexte caché hérité d'un clone parent (résumé cumulatif) est
        // extrait dans une section dédiée plutôt que sérialisé comme un
        // message utilisateur normal. Le LLM reçoit alors une entrée claire
        // `<inherited_context>` en tête, puis le reste de la conversation.
        if msg.content.trim_start().starts_with(CLONE_SUMMARY_PREFIX) {
            push_bounded(&mut inherited, msg.content.trim());
            push_bounded(&mut inherited, "\n\n");
            continue;
        }
        push_bounded(&mut out, &format!("\n<message role=\"{}\">\n", msg.role));
        push_bounded(&mut out, msg.content.trim());
        if let Some(thinking) = msg
            .thinking
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            push_bounded(&mut out, "\n<thinking>\n");
            push_bounded(&mut out, thinking.trim());
            push_bounded(&mut out, "\n</thinking>");
        }
        for tool in message_tools(msg) {
            serialize_tool(&mut out, tool);
        }
        push_bounded(&mut out, "\n</message>\n");
        if out.chars().count() >= MAX_SUMMARY_INPUT_CHARS {
            append_truncated_marker(&mut out);
            break;
        }
    }
    if !inherited.is_empty() {
        let prefix = format!(
            "<inherited_context>\n{}\n</inherited_context>\n\n",
            inherited.trim()
        );
        // On insère le bloc hérité en tête, dans le budget global.
        let remaining = MAX_SUMMARY_INPUT_CHARS.saturating_sub(out.chars().count());
        let prefix_bounded = take_chars(&prefix, remaining);
        out = format!("{prefix_bounded}{out}");
    }
    out
}

pub fn extract_traced_files(messages: &[AgentMessage]) -> (Vec<String>, Vec<String>) {
    let mut read_files = Vec::new();
    let mut modified_files = Vec::new();
    for msg in messages {
        for tool in message_tools(msg) {
            collect_tool_files(tool, &mut read_files, &mut modified_files);
        }
    }
    (read_files, modified_files)
}

fn serialize_tool(out: &mut String, tool: &ToolActivityRecord) {
    push_bounded(out, &format!("\n<tool name=\"{}\">\n", tool.name));
    if let Some(args) = &tool.args {
        push_bounded(out, "args: ");
        push_bounded(out, &args.to_string());
        push_bounded(out, "\n");
    }
    if !tool.summary.trim().is_empty() {
        push_bounded(out, "summary: ");
        push_bounded(out, tool.summary.trim());
        push_bounded(out, "\n");
    }
    if let Some(result) = tool.result.as_deref().or(tool.content.as_deref()) {
        push_bounded(out, "result: ");
        push_bounded(out, &take_chars(result, MAX_TOOL_RESULT_CHARS));
        push_bounded(out, "\n");
    }
    push_bounded(out, "</tool>");
}

fn collect_tool_files(
    tool: &ToolActivityRecord,
    read_files: &mut Vec<String>,
    modified_files: &mut Vec<String>,
) {
    let is_read = matches!(
        tool.name.as_str(),
        "read_file"
            | "grep"
            | "glob"
            | "list_dir"
            | "read_document"
            | "read_image"
            | "read_spreadsheet"
    );
    let is_write = matches!(
        tool.name.as_str(),
        "write_file"
            | "edit_file"
            | "process_image"
            | "write_document"
            | "write_spreadsheet"
            | "bash"
    );
    if is_read {
        add_arg_path(tool, read_files);
    }
    if is_write || !tool.affected_paths.is_empty() {
        add_arg_path(tool, modified_files);
        for path in &tool.affected_paths {
            add_path(modified_files, path);
        }
    }
}

fn add_arg_path(tool: &ToolActivityRecord, files: &mut Vec<String>) {
    let Some(args) = &tool.args else {
        return;
    };
    for key in ["path", "file", "output_path"] {
        if let Some(path) = args.get(key).and_then(|value| value.as_str()) {
            add_path(files, path);
        }
    }
}

fn add_path(files: &mut Vec<String>, path: &str) {
    let trimmed = path.trim();
    if trimmed.is_empty()
        || trimmed.contains('\0')
        || trimmed.chars().count() > 4096
        || files.len() >= MAX_TRACKED_FILES
        || files.iter().any(|seen| seen == trimmed)
    {
        return;
    }
    files.push(trimmed.to_string());
}

fn message_tools(message: &AgentMessage) -> Vec<&ToolActivityRecord> {
    if let Some(segments) = &message.segments {
        let tools: Vec<_> = segments
            .iter()
            .flat_map(|segment| segment.tools.iter())
            .collect();
        if !tools.is_empty() {
            return tools;
        }
    }
    message
        .tool_activities
        .as_deref()
        .unwrap_or_default()
        .iter()
        .collect()
}

fn push_bounded(out: &mut String, value: &str) {
    let remaining = MAX_SUMMARY_INPUT_CHARS.saturating_sub(out.chars().count());
    if remaining == 0 {
        return;
    }
    out.push_str(&take_chars(value, remaining));
}

fn take_chars(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn append_truncated_marker(out: &mut String) {
    let marker_len = TRUNCATED_MARKER.chars().count();
    let keep = MAX_SUMMARY_INPUT_CHARS.saturating_sub(marker_len);
    let trimmed = take_chars(out, keep);
    out.clear();
    out.push_str(&trimmed);
    out.push_str(TRUNCATED_MARKER);
}

#[cfg(test)]
#[path = "clone_summary_tests.rs"]
mod tests;
