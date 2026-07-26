use std::path::Path;

#[test]
fn compact_and_detailed_prompts_share_positive_subagent_guidance() {
    let compact = super::prompt_compact::build_with_behavior(Path::new("."), false, None, None);
    let detailed = super::prompt_detailed::build_with_behavior(Path::new("."), false, None, None);

    for prompt in [compact, detailed] {
        assert_eq!(prompt.matches("# Working with subagents").count(), 1);
        assert!(prompt.contains("After delegate_task, continue useful independent work"));
        assert!(prompt.contains("Do not repeatedly inspect subagents while they run"));
        assert!(prompt.contains("finish the turn without a tool call"));
        assert!(prompt.contains("When a coder report includes a pending change"));
        assert!(prompt.contains("captured in the coder's isolated workspace"));
        assert!(prompt.contains("not applied to the parent project"));
        assert!(prompt.contains("until apply_subagent_changes succeeds"));
        assert!(prompt.contains("Do not infer its state by checking whether files exist"));
        assert!(prompt.contains("inspect_subagent_changes"));
        assert!(prompt.contains("apply_subagent_changes"));
        assert!(prompt.contains("discard_subagent_changes"));
        assert!(prompt.contains("fix it yourself"));
        assert!(prompt.contains("An apply failure leaves the isolated change unresolved"));
        assert!(prompt.contains(
            "After integrating or recreating the intended result yourself, call \
discard_subagent_changes"
        ));
        assert!(prompt.contains(
            "Do not claim completion while the isolated change remains pending or conflicted"
        ));
        assert!(!prompt.contains("never redo the coder's work"));
    }
}

#[test]
fn dynamic_context_uses_natural_guidance_without_lock_vocabulary() {
    let context = super::subagent_orchestration_context::build_gate_content(1, false);

    for forbidden in ["Final answer is locked", "Keep the stream active", "blocked"] {
        assert!(!context.contains(forbidden), "forbidden text: {forbidden}");
    }
    assert!(!context.contains("final_answer_allowed"));
    assert!(context.contains("Terminal reports arrive automatically"));
    assert!(context.contains(
        "use it for useful independent work and defer the overall summary"
    ));
    assert!(context.contains("finish this turn without a tool call"));
}

#[test]
fn detailed_prompt_does_not_add_interactive_choice_section() {
    let prompt = super::prompt_detailed::build_with_behavior(
        Path::new("."),
        false,
        None,
        None,
    );

    assert!(!prompt.contains("# Interactive choices"));
}
