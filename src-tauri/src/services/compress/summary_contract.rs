use subtle::ConstantTimeEq;

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
    if output.tool_call_count != 0 {
        return Err(invalid("compression_summary_tool_call"));
    }
    if output.truncated {
        return Err(invalid("compression_summary_truncated"));
    }
    if output.cancelled {
        return Err(invalid("compression_summary_cancelled"));
    }
    let trimmed = output.content.trim();
    if !trimmed.starts_with("<summary>") {
        return Err(invalid("compression_summary_missing_open_tag"));
    }
    if !trimmed.ends_with("</summary>") {
        if trimmed.contains("</summary>") {
            return Err(invalid("compression_summary_trailing_text"));
        }
        return Err(invalid("compression_summary_missing_close_tag"));
    }
    let content = normalized_envelope_body(trimmed)?;
    if content.is_empty() {
        return Err(invalid("compression_summary_empty"));
    }
    let mut cursor = 0;
    for (index, section) in required_sections().into_iter().enumerate() {
        let offset = content[cursor..].find(section).ok_or_else(|| {
            invalid(match index {
                0 => "compression_summary_missing_section_1",
                1 => "compression_summary_missing_section_2",
                2 => "compression_summary_missing_section_3",
                3 => "compression_summary_missing_section_4",
                4 => "compression_summary_missing_section_5",
                5 => "compression_summary_missing_section_6",
                6 => "compression_summary_missing_section_7",
                7 => "compression_summary_missing_section_8",
                _ => "compression_summary_missing_section_9",
            })
        })?;
        cursor = cursor.saturating_add(offset).saturating_add(section.len());
    }
    let content = super::compression_redaction::redact_checkpoint_text(&content);
    let estimated = crate::services::token_counting::estimate_text_tokens(&content);
    if estimated == 0 {
        return Err(invalid("compression_summary_empty_after_redaction"));
    }
    if estimated > maximum_tokens as usize {
        return Err(invalid("compression_summary_over_budget"));
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

fn invalid(code: &'static str) -> String {
    code.to_string()
}

fn normalized_envelope_body(output: &str) -> Result<String, String> {
    let open = "<summary>";
    let close = "</summary>";
    let body = output
        .strip_prefix(open)
        .and_then(|value| value.strip_suffix(close))
        .ok_or_else(|| invalid("compression_summary_invalid_envelope"))?;

    if let Some(first_close) = body.find(close) {
        let after_close = first_close.saturating_add(close.len());
        if let Some(second_open_offset) = body[after_close..].find(open) {
            let second_open = after_close.saturating_add(second_open_offset);
            if body[after_close..second_open].trim().is_empty() {
                let first = body[..first_close].trim();
                let second = body[second_open.saturating_add(open.len())..].trim();
                if first.len() == second.len()
                    && bool::from(first.as_bytes().ct_eq(second.as_bytes()))
                {
                    return Ok(escape_inner_envelope_markers(first));
                }
                return Err(invalid("compression_summary_duplicate_distinct"));
            }
        }
    }

    Ok(escape_inner_envelope_markers(body.trim()))
}

fn escape_inner_envelope_markers(body: &str) -> String {
    body.replace("<summary>", "&lt;summary&gt;")
        .replace("</summary>", "&lt;/summary&gt;")
}
