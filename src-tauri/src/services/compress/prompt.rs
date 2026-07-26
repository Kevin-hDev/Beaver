pub fn build_compression_prompt(custom_instructions: Option<&str>) -> String {
    let mut prompt = String::from(PREAMBLE);
    prompt.push_str(BASE_PROMPT);
    if let Some(instructions) = custom_instructions {
        prompt.push_str("\n\nAdditional Instructions:\n");
        prompt.push_str(instructions);
    }
    prompt.push_str(TRAILER);
    prompt
}

const PREAMBLE: &str = "\
CRITICAL: Respond with TEXT ONLY. Do NOT call any tools.

- Do NOT use Read, Bash, Grep, Glob, Edit, Write, or ANY other tool.
- You already have all the context you need in the conversation above.
- Tool calls will be REJECTED and will waste your only turn — you will fail the task.
- Your entire response must be plain text inside a single <summary> block.

";

const BASE_PROMPT: &str = "\
Your task is to create a detailed summary of the conversation so far, paying close \
attention to the user's explicit requests and your previous actions. This summary should \
be thorough in capturing technical details, code patterns, and architectural decisions \
that would be essential for continuing development work without losing context.

Your summary MUST include the following sections:

1. Primary Request and Intent: Capture all of the user's explicit requests and intents \
in detail.
2. Key Technical Concepts: List all important technical concepts, technologies, and \
frameworks discussed.
3. Files and Code Sections: Enumerate specific files and code sections examined, modified, \
or created. Include short snippets only when essential and explain why each read or edit matters.
4. Errors and Fixes: List all errors encountered and how they were resolved. Pay special \
attention to specific user feedback.
5. Problem Solving: Document problems solved and any ongoing troubleshooting efforts.
6. All User Messages: List ALL user messages that are not tool results. These are critical \
for understanding feedback and changing intent.
7. Pending Tasks: Outline any pending tasks that have explicitly been asked to work on.
8. Current Work: Describe in detail precisely what was being worked on immediately before \
this summary request. Include file names and code snippets where applicable.
9. Next Step: State the next concrete action related to the most recent work. Include \
verbatim citations where possible.

";

const TRAILER: &str = "\n\
REMINDER: Do NOT call any tools. Respond with plain text only inside one <summary> block. \
Tool calls will be rejected and you will fail the task.";

pub fn format_summary_message(summary: &str, suppress_follow_up: bool) -> String {
    let mut msg = String::from(
        "This session is being continued from a previous conversation that ran out of \
         context. The summary below covers the earlier portion of the conversation.\n\n",
    );
    msg.push_str(summary);
    if suppress_follow_up {
        msg.push_str(
            "\n\nContinue the conversation from where it left off without asking the user \
             any further questions. Resume directly — do not acknowledge the summary, do not \
             recap what was happening, do not preface with \"I'll continue\" or similar.",
        );
    }
    msg
}

pub fn extract_summary(response: &str) -> String {
    if let Some(start) = response.find("<summary>") {
        let content_start = start + "<summary>".len();
        if let Some(end) = response[content_start..].find("</summary>") {
            return response[content_start..content_start + end]
                .trim()
                .to_string();
        }
    }
    response.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_contains_all_sections() {
        let prompt = build_compression_prompt(None);
        assert!(prompt.contains("1. Primary Request"));
        assert!(prompt.contains("2. Key Technical"));
        assert!(prompt.contains("3. Files and Code"));
        assert!(prompt.contains("4. Errors and Fixes"));
        assert!(prompt.contains("5. Problem Solving"));
        assert!(prompt.contains("6. All User Messages"));
        assert!(prompt.contains("7. Pending Tasks"));
        assert!(prompt.contains("8. Current Work"));
        assert!(prompt.contains("9. Next Step"));
    }

    #[test]
    fn prompt_has_tool_enforcement() {
        let prompt = build_compression_prompt(None);
        assert!(prompt.contains("CRITICAL"));
        assert!(prompt.contains("Do NOT call any tools"));
        assert!(prompt.contains("REMINDER"));
        assert!(prompt.contains("REJECTED"));
    }

    #[test]
    fn prompt_has_analysis_block_instruction() {
        let prompt = build_compression_prompt(None);
        assert!(!prompt.contains("<analysis>"));
        assert!(prompt.contains("<summary>"));
    }

    #[test]
    fn prompt_with_custom_instructions() {
        let prompt = build_compression_prompt(Some("Focus on Rust code"));
        assert!(prompt.contains("Focus on Rust code"));
        assert!(prompt.contains("Additional Instructions"));
    }

    #[test]
    fn format_summary_wraps_correctly() {
        let msg = format_summary_message("My summary here", false);
        assert!(msg.contains("My summary here"));
        assert!(msg.contains("previous conversation"));
        assert!(!msg.contains("without asking"));
    }

    #[test]
    fn format_summary_auto_suppresses_questions() {
        let msg = format_summary_message("Summary", true);
        assert!(msg.contains("without asking"));
        assert!(msg.contains("Resume directly"));
    }

    #[test]
    fn extract_summary_with_tags() {
        let response =
            "<analysis>Internal thinking</analysis>\n<summary>The real summary</summary>";
        assert_eq!(extract_summary(response), "The real summary");
    }

    #[test]
    fn extract_summary_no_tags_fallback() {
        let response = "No tags here";
        assert_eq!(extract_summary(response), "No tags here");
    }

    #[test]
    fn extract_summary_strips_whitespace() {
        let response = "<summary>\n  Spaced summary\n  </summary>";
        assert_eq!(extract_summary(response), "Spaced summary");
    }
}
