# 01 - Initialize or resume

You validate the measurable goal, reconcile prior state, map the journey, and prepare one durable tracking file.

## Input

- Accept the user's objective, requested effects, constraints, and any success command or predicate.
- Use the current project evidence and an optional existing tracking-file path.

## Output

- Return a reconciled existing run or a new `pending` tracking record.
- Return a journey map, prerequisite inventory, current-work snapshot, and unresolved blockers.

## Process

1. You derive a stable task slug from the objective and validate every supplied path; you reject traversal, links escaping the project, and ambiguous roots.
2. You select the tracking destination in this order: an explicit user path, an established project-local task-log convention, then the announced fallback `.persistent-runs/<task-slug>.md`.
3. You look for an existing record only at the selected destination. You never merge records merely because names look similar.
4. You load the entire record when it exists, validate its required fields, and reconcile it with the current branch, working-tree state, relevant files, available tools, and prior evidence.
5. You preserve changes made after the last attempt. You mark drift in the record and revise the journey rather than overwriting or reverting user work.
6. You handle the recorded state precisely:
   - You report `completed` with its evidence and stop unless the user explicitly requests fresh revalidation.
   - You resume `pending` or `in-progress` from the first unchecked step after reconciliation.
   - You resume `blocked` only when the recorded blocker is now resolved or the user supplies the missing input, authority, or boundary extension.
7. You research relevant project documentation and current official guidance before you map an approach that depends on external tools, APIs, version-specific behavior, or an unfamiliar mechanism. You do not substitute recalled defaults for guidance that may have changed.
8. You reject ambiguous or subjective outcomes until they become observable. You accept a deterministic predicate only when another executor can reproduce it.
9. You inventory required tools, data, secrets, services, and external access. You label each item `available`, `obtainable within contract`, or `blocked` without exposing secret values.
10. You inventory the relevant environment and versions, applicable project instructions, and expected artifacts to read, create, edit, or remove. You include removal only when the original request explicitly requires it.
11. You map ordered steps, dependencies, verification points, risks, and likely alternative hypotheses. You add a compact decision diagram only when the journey has a meaningful branch that a table would hide.
12. You create a new record from [the tracking template](../assets/tracking-template.md) only after the path, goal, predicate, and prerequisites are known. You leave `completion: unverified`.
13. You proceed to the action contract. You do not start an attempt while a prerequisite needed for the first step remains blocked.

## Stop conditions

- You stop when the goal cannot be made observable without user clarification.
- You stop when the selected record is malformed, conflicts with another task, or cannot be reconciled safely.
- You stop when required input, access, or authority is missing.
- You stop without mutation when an existing record is already `completed` and fresh revalidation was not requested.

## Test

- You confirm the selected path stays inside the intended project and the record has all required fields.
- You confirm a new run starts as `pending` with `completion: unverified` and an append-only empty log.
- You confirm a resumed run preserves current user work and identifies the exact next unchecked step.
- You confirm every prerequisite has one explicit availability state.
