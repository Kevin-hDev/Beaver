# 02 - Evaluate Stop Conditions

You decide whether to continue correction or hand control back using the fixed stop-order contract.

## Input

- Use the complete feedback collection, validated configuration, linked-ticket states, and durable iteration history.
- Use the ordered review stop conditions reference.

## Output

- Return `stop` or `continue`, one exact reason, whether finalization is required, and the evidence for the first matching rule.

## Process

1. You return `stop` with `blocked-state` when the linked ticket carries the configured blocked state.
2. You return `stop` with `iteration-limit` when completed correction iterations have reached the configured maximum.
3. You treat human feedback in the first iteration as the requested correction input and never stop merely because it is human-authored.
4. You return `stop` with `new-human-feedback` after the first correction iteration when any human comment is strictly newer than the recorded start of the preceding iteration.
5. You return `stop` with `no-unaddressed-feedback` when no eligible unaddressed human feedback remains.
6. You return `continue` only when none of the prior rules matches and at least one eligible human comment remains.
7. You record only the first matching reason. You do not reorder conditions based on convenience or adapter behavior.
8. You set finalization required on every stop decision and correction required only on `continue`.

## Stop conditions

- You stop evaluation when feedback collection is incomplete or iteration timestamps cannot be trusted.
- You stop when the configured maximum is missing, invalid, or unbounded.
- You stop without correction whenever one ordered stop rule matches.

## Test

- You confirm blocked state wins over every other simultaneous condition.
- You confirm the iteration limit wins over new human feedback.
- You confirm first-iteration human feedback continues when it is eligible.
- You confirm new human feedback after the first iteration stops before correction.
- You confirm an empty eligible set finalizes benignly with `no-unaddressed-feedback`.
