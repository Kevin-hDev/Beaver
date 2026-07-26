const MAX_SYSTEM_PROMPT_BYTES: usize = 64 * 1024;

pub fn normalize_prompt(prompt: &str) -> Result<Option<String>, String> {
    let normalized = prompt.trim();
    if normalized.is_empty() {
        return Ok(None);
    }
    if normalized.len() > MAX_SYSTEM_PROMPT_BYTES
        || normalized.contains('\0')
        || normalized.contains("\"\"\"")
    {
        return Err("ollama-system-prompt-invalid".into());
    }
    Ok(Some(normalized.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsafe_or_oversized_prompts() {
        assert!(normalize_prompt("hello\0world").is_err());
        assert!(normalize_prompt("contains \"\"\" delimiter").is_err());
        assert!(normalize_prompt(&"x".repeat(MAX_SYSTEM_PROMPT_BYTES + 1)).is_err());
    }
}
