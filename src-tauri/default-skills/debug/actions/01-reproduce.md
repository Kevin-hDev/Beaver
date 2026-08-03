# 01 - Reproduce

You establish a minimal repeatable failure and localize the code path that causes it.

## Input

- Accept a bug description, failing test, error, issue, or reproducible behavior.

## Output

- Return expected behavior, actual behavior, minimal trigger, reproduction result, affected path, evidence for the localized cause, and any explicitly requested delivery baseline.

## Process

1. **Validate.** You validate every path and reject unreadable or escaping sources. You inspect at most 100,000 inline characters or 256 KiB from a referenced file per input batch and continue later batches until the complete issue source is covered.
2. **Read rules.** You load only the project instructions that apply to the affected files and checks.
3. **Establish baseline.** You separate the reported defect from unrelated existing failures and user changes.
4. **Select delivery endpoint.** When the user explicitly requests the full ticket-to-pull-request outcome, you validate the repository, ticket provider, code-host provider, remote, default branch, and publication authority. You create one issue and one dedicated fix branch before test or production edits. Otherwise, you keep the workflow local and uncommitted.
5. **Reproduce in batches.** You run the smallest safe trigger at most three times per batch. You record exact sanitized output and whether the failure is consistent. When a batch is inconclusive, you preserve it and continue through `04-investigate-cause` instead of declaring a terminal failure.
6. **Trace.** You follow the failing path through the smallest necessary callers, data transformations, configuration, and boundaries.
7. **Compare.** You inspect a nearby working path, recent relevant changes, and the complete error chain when they can distinguish the cause.
8. **Localize.** You name the source-level cause only when the reproduction and code evidence agree.

## Stop conditions

- You stop without edits when the source or trigger is missing or unsafe.
- You continue with `04-investigate-cause` when the defect remains unreliable or the causal boundary remains unclear.
- You remove secrets and sensitive values from evidence and user-visible errors.

## Test

- The reported reproduction distinguishes expected from actual behavior and can be repeated.
- The localized cause cites a real path and explains how it produces the symptom.
- No production file changes during reproduction.
- Full-delivery mode has one verified issue identifier and a dedicated non-default fix branch before test edits; local mode changes no external state.
