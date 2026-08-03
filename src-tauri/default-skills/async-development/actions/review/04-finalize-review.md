# 04 - Finalize Review

You finalize the review loop idempotently with a durable result and only the authorized, verified external effects.

## Input

- Use the exact change request and linked ticket, stop decision, iteration history, review lock, configuration, effect contract, and adapter records.
- Use the run record and review summary templates.

## Output

- Return the durable review audit path and digest, stop reason, final state, summary and marker identities, result artifact, and verification status.
- Return `blocked` or `incomplete` instead of success when any required critical effect is unverified.

## Process

1. You compose a bounded review record with run id, idempotency key, ticket and change-request identities, stop reason, append-only iterations, tests, replies, resolutions, timestamps, and sanitized error.
2. You validate the project audit directory and write the review record atomically. You re-read it and verify its digest before external effects.
3. You write an integration result artifact only when the verified remote contract requires one. You use and validate the real provider schema.
4. You render the review summary template from observed data. You search all relevant bounded comment pages for its idempotency key before posting. You return `incomplete` without posting when the configured bound is reached before duplicate detection proves exhaustion.
5. You post the summary and completion marker only when their exact comment effects are authorized. You verify each identifier and content independently.
6. You transition the linked ticket from this run's working lock to blocked state for `blocked-state`, `critical-failure`, or an actual critical effect failure, and to awaiting-review for `iteration-limit`, `new-human-feedback`, or `no-unaddressed-feedback`. You require exact lifecycle authority, matching owner and revision, removal of working state, and an independent read of the terminal state.
7. You invoke a deterministic integration finalizer only when its real interface is available and every effect it performs is authorized. You independently inspect its result.
8. You record every finalization observation atomically without altering prior iteration events.
9. You make reruns idempotent: you reuse matching verified summary and marker effects, refuse conflicting keys, and perform only missing authorized effects.
10. You return success only when the durable audit and every required state, result, summary, and marker effect are verified.
11. You never abandon this run's working lock after the durable audit exists. When any required result, summary, marker, reaction, reply, resolution, or other post-audit effect fails, you preserve that original failure, stop all non-cleanup effects, conditionally transition only this run's lock to blocked, verify the transition, and append the observation. A successful cleanup never converts the failed finalization into success.

## Stop conditions

- You stop before remote finalization when the durable review record or digest is invalid.
- You fail closed when a required result artifact, summary, marker, lifecycle transition, or verification fails. You allow only the mandatory run-owned lock cleanup after another critical effect fails.
- You stop after the durable audit and verified working-to-blocked lock closure when the correction iteration recorded base-branch drift. You perform no other remote effect.

## Test

- You confirm the audit record is durable, append-only, and matches its digest.
- You confirm a repeated finalization with the same key creates no duplicate summary, marker, or state transition.
- You confirm each stop reason maps to the configured terminal state exactly.
- You confirm one failed critical comment or lifecycle operation prevents a successful final status.
- You confirm a failed critical effect preserves its error, releases only this run's working lock to blocked when possible, and never reports successful finalization.
