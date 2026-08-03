# 04 - Generate Local Runner

You generate a local one-cycle entry point only when a real non-interactive runtime interface is available.

## Input

- Use the confirmed configuration, effect contract, and detection report.
- Use the verified runtime invocation and tracker adapter interfaces.

## Output

- Return the generated runner path, dry-run contract, executable state, and runtime validation evidence.
- Return `skipped` for remote-only execution or `unsupported` when invocation cannot be verified.

## Process

1. You skip with a recorded reason when local execution is not selected.
2. You require explicit authority to write or replace the exact local runner path.
3. You require a real non-interactive runtime entry point that can invoke this skill with an exact action and item identifier. You do not guess flags or use a safety-bypass option.
4. You define eligibility from observed lifecycle state: you exclude working and blocked items, route a ready item to `action=run`, and route a review item to `action=review` only when its uniquely related change request is observable. You refuse an ambiguous item with conflicting triggers or relationships instead of choosing a target silently.
5. You generate a dry-run path that lists at most one next eligible item and performs no model invocation or external mutation.
6. You generate a normal path that selects one eligible item, invokes exactly one sub-flow, returns its exit status, and records a continuation cursor when more work exists. You exit successfully without a runtime invocation when no eligible item exists.
7. You pass all commands as validated argument arrays. You quote data safely and never evaluate ticket content as code.
8. You bound page size, pages, output, and execution time from configuration.
9. You preserve an existing runner unless the user confirms replacement after reviewing a diff.
10. You write atomically, set executable permissions only when appropriate, and leave the file unstaged.
11. You execute the dry run and prove it performs no mutations through before-and-after observations.

## Stop conditions

- You stop when the runtime invocation, tracker query, or one-item guarantee cannot be verified.
- You stop when dry-run isolation fails or an existing file lacks replacement authority.
- You stop when the runner would need a secret value embedded in the file or conversation.

## Test

- You confirm dry run reports at most one item and performs no external mutation or model call.
- You confirm working and blocked items are skipped, and an empty eligible queue makes no runtime call.
- You confirm normal mode can dispatch only `action=run` or `action=review` with a validated identifier.
- You confirm the runner contains no unbounded queue loop, unsafe evaluation, or safety-bypass flag.
- You confirm a non-zero child result propagates as a non-zero runner result.
