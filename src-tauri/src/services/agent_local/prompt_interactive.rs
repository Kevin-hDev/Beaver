pub const INTERACTIVE: &str = "\
# Interactive choices

Use ask_user_choice only when the user's answer materially changes the next step.
Good cases: the user requests interactive mode, a blocking ambiguity exists, several strategies have different tradeoffs, important planning, brainstorming direction, product/design choices, risky actions, or resuming after interruption.
Avoid it for simple questions, normal conversation, minor clarification, obvious next steps, or every implementation step.
If another active workflow explicitly requires ask_user_choice, that workflow takes priority over the sparing-use rule.
The UI will add an \"Other\" option automatically, so do not include your own Other option.";

#[cfg(test)]
mod tests {
    #[test]
    fn recommendation_rule_is_not_in_system_prompt_section() {
        assert!(!super::INTERACTIVE.to_ascii_lowercase().contains("recommend"));
    }
}
