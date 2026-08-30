#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedSummary {
    pub content: String,
    pub estimated_tokens: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryRawOutput {
    pub content: String,
    pub tool_call_count: usize,
    pub truncated: bool,
    pub cancelled: bool,
}

pub fn validate(output: SummaryRawOutput, maximum_tokens: u32) -> Result<ValidatedSummary, String> {
    if output.tool_call_count != 0 || output.truncated || output.cancelled {
        return Err(invalid());
    }
    let trimmed = output.content.trim();
    if !trimmed.starts_with("<summary>")
        || !trimmed.ends_with("</summary>")
        || trimmed.matches("<summary>").count() != 1
        || trimmed.matches("</summary>").count() != 1
    {
        return Err(invalid());
    }
    let content = trimmed
        .strip_prefix("<summary>")
        .and_then(|value| value.strip_suffix("</summary>"))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(invalid)?;
    let mut cursor = 0;
    for section in required_sections() {
        let offset = content[cursor..].find(section).ok_or_else(invalid)?;
        cursor = cursor.saturating_add(offset).saturating_add(section.len());
    }
    let content = super::compression_redaction::redact_checkpoint_text(content);
    let estimated = crate::services::token_counting::estimate_text_tokens(&content);
    if estimated == 0 || estimated > maximum_tokens as usize {
        return Err(invalid());
    }
    Ok(ValidatedSummary {
        content,
        estimated_tokens: estimated.min(u32::MAX as usize) as u32,
    })
}

pub fn required_sections() -> [&'static str; 9] {
    [
        "1. Primary Request and Intent",
        "2. Key Technical Concepts",
        "3. Files and Code Sections",
        "4. Errors and Fixes",
        "5. Problem Solving",
        "6. User Intent and Corrections",
        "7. Pending Tasks",
        "8. Current Work",
        "9. Next Step",
    ]
}

fn invalid() -> String {
    "compression_summary_invalid".to_string()
}
