# 03 - Generate Integration

You generate a remote event integration only from a verified provider format and the confirmed effect contract.

## Input

- Use the confirmed configuration, effect contract, and detection report.
- Use the provider's inspected schema or official current integration format.

## Output

- Return the generated integration path, adapter identity, validation evidence, and exact dispatched sub-flows.
- Return `skipped` for a local-only configuration or `unsupported` when the real format is unavailable.

## Process

1. You skip with a recorded reason when remote execution is not selected.
2. You require explicit authority to write the exact integration path. You preserve an existing file unless replacement was separately authorized after a diff.
3. You read the integration-generation guide and the generic integration contract asset.
4. You construct the file from the inspected provider schema. You include only the minimum permissions required by authorized effects.
5. You filter configured ready, review, and mention events and reject automated trigger authors where supported. You resolve the event target before dispatch: you route a change-request event to `action=review`; you route a ticket with no linked active change request to `action=run` only for run intent; you route review intent on a ticket to its uniquely linked active change request; and you return an explicit no-op when review has no reviewable change or run intent would duplicate an active change request. You reject simultaneous or ambiguous intents unless the confirmed configuration defines one deterministic precedence.
6. You key concurrency or idempotency to the source ticket or change-request identity and disable cancellation that could leave an acquired lock unresolved.
7. You reference secure credentials by configured name only. You never place a value, example token, or conversational secret in the file.
8. You require one-item execution, a bounded result artifact, a deterministic finalizer, and verified failure propagation for critical state, comment, audit, and test operations. You make the finalizer independently observe the ticket, change request, and result artifact instead of trusting the worker's outcome claim.
9. You make finalization clear the working state and apply awaiting-review only after observed success or verified recovery, or blocked after observed failure. You exclude error-suppression patterns for critical operations and make state changes, audit records, and completion markers idempotent through stable run and completion keys.
10. You write the file atomically, then validate it with the provider's actual parser or validation endpoint.
11. You leave the file unstaged. You record any missing capability as `unsupported` and do not emit a speculative adapter.

## Stop conditions

- You stop before writing when the integration format, validator, concurrency primitive, or secure-reference mechanism is unavailable.
- You stop when replacement authority, path authority, or a required finalization capability is absent.
- You stop and preserve the previous file when syntax or provider validation fails.

## Test

- You confirm the integration dispatches exactly one selected sub-flow per event.
- You confirm duplicate run intent, review without a reviewable change, and ambiguous linkage produce explicit no-op or blocked outcomes rather than a second implementation or a guessed target.
- You confirm concurrency uses the external item identity and every collection is bounded.
- You confirm no secret value or unsafe bypass flag appears in the generated file.
- You confirm the actual provider validator accepts the final artifact.
