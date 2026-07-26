const SUMMARY_OPEN: &str = "<memory_summary scope=\"";
const SUMMARY_CLOSE: &str = "\n</memory_summary>";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoryContextUsage {
    pub prompt_tokens: usize,
    pub summary_tokens: usize,
}

impl MemoryContextUsage {
    pub fn from_section(section: &str) -> Self {
        let total = estimate(section);
        let summary_tokens = estimate(&meaningful_summary_text(section)).min(total);
        Self {
            prompt_tokens: total.saturating_sub(summary_tokens),
            summary_tokens,
        }
    }

    pub fn total(self) -> usize {
        self.prompt_tokens.saturating_add(self.summary_tokens)
    }
}

fn meaningful_summary_text(section: &str) -> String {
    let mut remaining = section;
    let mut output = String::new();
    while let Some(open_index) = remaining.find(SUMMARY_OPEN) {
        let after_marker = &remaining[open_index + SUMMARY_OPEN.len()..];
        let Some(body_index) = after_marker.find(">\n") else {
            break;
        };
        let body_and_tail = &after_marker[body_index + 2..];
        let (body, next) = match body_and_tail.find(SUMMARY_CLOSE) {
            Some(close_index) => (
                &body_and_tail[..close_index],
                &body_and_tail[close_index + SUMMARY_CLOSE.len()..],
            ),
            None => (body_and_tail, ""),
        };
        if body.lines().any(|line| line.trim_start().starts_with("- ")) {
            output.push_str(body);
        }
        if next.is_empty() {
            break;
        }
        remaining = next;
    }
    output
}

fn estimate(content: &str) -> usize {
    crate::services::token_counting::estimate_text_tokens(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separates_real_summary_from_memory_protocol() {
        let project = "# Résumé mémoire\n\n- **Tokens CSS** — Réutiliser les tokens.\n";
        let summaries = super::super::memory_prompt::format_summaries("", project);
        let section = format!(
            "<memory_context>\nrules\n{summaries}</memory_context>"
        );

        let usage = MemoryContextUsage::from_section(&section);

        assert_eq!(usage.summary_tokens, estimate(project));
        assert_eq!(usage.total(), estimate(&section));
    }

    #[test]
    fn ignores_empty_generated_summary() {
        let section = "<memory_context>\n\
            <memory_summary scope=\"global\">\n# Résumé mémoire\n\n</memory_summary>\n\
            </memory_context>";

        let usage = MemoryContextUsage::from_section(section);

        assert_eq!(usage.summary_tokens, 0);
        assert_eq!(usage.prompt_tokens, estimate(section));
    }

    #[test]
    fn counts_a_summary_truncated_inside_its_first_entry() {
        let partial = "# Résumé mémoire\n\n- **Préférence** — réponse concise";
        let section = format!(
            "<memory_context>\nrules\n\
             <memory_summary scope=\"project\">\n{partial}"
        );

        let usage = MemoryContextUsage::from_section(&section);

        assert_eq!(usage.summary_tokens, estimate(partial));
        assert_eq!(usage.total(), estimate(&section));
    }
}
