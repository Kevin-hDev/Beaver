# Implementation Guardrails

Use these rules while you execute an approved plan.

## Preserve the workspace

- You inspect version-control status before and after every phase.
- You treat pre-existing changes as user-owned.
- You avoid files with overlapping user edits unless you can preserve them exactly.
- You do not restore, reset, clean, stash, or rewrite unrelated work.

## Detect drift

Report `replan needed` and stop when any condition holds:

- The required public behavior differs from the approved acceptance checks.
- The repository architecture makes a planned boundary invalid.
- A new dependency, migration, data loss risk, public contract, or security decision becomes necessary.
- The requested outcome cannot fit inside the planned files or phases without material expansion.

Do not treat a corrected candidate file path or a small implementation detail as drift when the approved behavior and architecture remain unchanged.

## Detect blockers

Report `blocked` and stop when continuation requires:

- A human login, hardware confirmation, payment, or interactive challenge.
- A secret or permission you cannot access through the approved tool path.
- An unavailable required service or external decision.
- A destructive action the user did not explicitly request.

## Repair failures

- You distinguish failures caused by the current change from pre-existing failures.
- You repair only current-change failures inside scope.
- You preserve raw failure evidence without exposing secrets or internal sensitive values.
- You keep each repair batch to three attempts on the same failing gate.
- After a failed batch, you preserve the attempted causes, edits, and sanitized evidence, then continue with a new batch of revised hypotheses.
- You stop only for a human-only blocker, unsafe expansion, unavailable required evidence, or an explicit user stop; the batch limit alone is not a blocker.
