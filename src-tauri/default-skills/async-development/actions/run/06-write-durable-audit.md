# 06 - Write Durable Audit

You persist the complete run result and finalize only the lifecycle effects that are authorized and independently verifiable.

## Input

- Use the poll and dependency record, optional lock and delegation observations, validated configuration, effect contract, and adapter capability records.
- Use the run record template and validated project audit directory.

## Output

- Return the durable audit path and digest, result-artifact path when required, every finalization effect, continuation state, and final outcome.
- Return `blocked` for any critical audit, state, comment, or verification failure.

## Process

1. You merge timestamps, item and change-request identities, optional lock data, revisions, tests, effects, observations, errors, and continuation data into the run record template.
2. You bound and sanitize error text, redact sensitive data, preserve the effect contract digest, and keep all prior append-only events.
3. You validate the audit directory inside the project and write the record through a temporary sibling and atomic rename.
4. You compute a digest and re-read the file before any external finalization.
5. You write a provider result artifact only when the verified integration contract requires it. You use its actual schema and validate it before handoff.
6. You transition working state to awaiting-review after a completed or safe partial result, or to blocked after a verified failure, including default-branch drift, only when the exact transition is authorized. You bind the transition to this run's lock revision and verify ownership, removal of working state, terminal state, and new revision. After drift, this lock closure is the only permitted remote effect.
7. You post an authorized bounded completion or blocked marker with a stable idempotency key only when delegation did not report default-branch drift. You first search all relevant pages within the configured bound and fail closed if duplicate detection is incomplete.
8. You invoke an external deterministic finalizer only when its real interface is available, the effect contract covers every effect, and delegation did not report default-branch drift. You independently observe its state, audit, and marker results.
9. You update the durable record atomically with finalization observations. You never rewrite prior verified events.
10. You return final success only when every required critical effect is verified. You retain a precise resumable continuation for an incomplete non-destructive optional effect.

## Stop conditions

- You stop before external finalization when the durable audit write or digest check fails.
- You fail closed when a required state, comment, result artifact, or independent verification fails.
- You stop after the durable audit and verified conditional working-to-blocked lock closure when delegation reported default-branch drift. You perform no other remote effect.

## Test

- You confirm the audit exists durably inside the project and matches its returned digest.
- You confirm a second finalization with the same idempotency key creates no duplicate marker or audit event.
- You confirm one failed required state or comment effect prevents a successful final outcome.
- You confirm a default-drift record retains exact evidence, closes only this run's lock into verified blocked state, and performs no comment, marker, change-request, or publication finalization. You allow a required result artifact to record the blocked outcome without causing a remote mutation.
