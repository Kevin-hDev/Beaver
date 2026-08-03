# 09 - Configure Scheduling

You configure an optional execution cadence only through a verified scheduler and exact scheduling authority.

## Input

- Use the confirmed configuration, effect contract, detection report, and generated local or remote integration.
- Accept an explicit choice among event-driven, manual, local supervisor, runtime scheduler, or service scheduler paths that are actually supported.

## Output

- Return the selected path, cadence, overlap policy, schedule identifier, enabled state, disable operation, and verification evidence.
- Return `manual`, `event-driven`, `skipped`, or `unsupported` without creating a schedule when appropriate.

## Process

1. You present event-driven and manual execution without requiring a schedule. You present only detected schedulers for periodic execution.
2. You explain cadence, resource, persistence, overlap, machine-availability, and quota tradeoffs supported by real adapter evidence.
3. You require explicit authority for the chosen scheduler, repository target, command or dispatch, cadence, and initial enabled state.
4. You require a positive minimum-safe cadence, one-item invocation, overlap prevention, bounded runtime, failure propagation, and a documented disable operation.
5. You generate a local supervisor artifact only from the actual platform format. You preserve an existing artifact without replacement authority.
6. You create a runtime or service schedule only through the verified scheduler adapter. You never invent a directive or raw automation syntax.
7. You capture a stable schedule identifier and independently inspect it for target, cadence, enabled state, and overlap policy.
8. You keep configuration valid with scheduling disabled when the user chooses manual or declines creation.
9. You leave generated scheduling files unstaged and record the exact continuation step when activation remains manual.

## Stop conditions

- You stop before creation when scheduling authority, a verified adapter, disable support, or overlap control is missing.
- You stop when the requested cadence violates a documented minimum or bounded-resource policy.
- You stop when the created schedule cannot be independently inspected.

## Test

- You confirm manual selection creates no schedule and returns the exact one-cycle invocation.
- You confirm an authorized schedule invokes at most one item per cycle and forbids overlap.
- You confirm the returned identifier resolves to the expected target and cadence.
- You confirm the disable operation exists and does not require a secret value in the conversation.
