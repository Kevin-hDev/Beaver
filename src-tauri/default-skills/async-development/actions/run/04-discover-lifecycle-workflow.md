# 04 - Discover Lifecycle Workflow

You resolve exactly one complete development workflow by advertised capability before delegation.

## Input

- Use the runtime capability catalog, locked ticket, configuration, effect contract, and pending audit record.
- Use the required plan, implementation, test, review, commit, and change-request capability set.

## Output

- Return one selected workflow identity, match evidence, supported effects, and invocation schema.
- Return the verified lock-release result when no suitable workflow exists.

## Process

1. You enumerate a bounded runtime capability catalog and inspect descriptions or schemas for the complete lifecycle capability set.
2. You reject candidates that cannot accept the ticket request, honor the recorded effect boundary, or expose an invocation interface you can verify.
3. You rank multiple valid candidates by explicit orchestration coverage, exact effect-contract support, then a stable deterministic identity order.
4. You select exactly one workflow and record why it satisfies every required capability.
5. You perform no delegation when no candidate qualifies.
6. You release the lock through one verified conditional transition only when lock release is already covered by the lifecycle state authority. You restore ready state and remove working state only if the current lock still belongs to this run.
7. You post a missing-capability comment only when that exact comment is authorized and idempotent.
8. You append the blocked reason and every attempted effect to the durable audit record.

## Stop conditions

- You stop when no complete workflow or verified invocation schema exists.
- You stop when candidate selection remains ambiguous after deterministic tie-breaks.
- You fail closed when an authorized lock release or comment cannot be verified.

## Test

- You confirm a catalog with one complete workflow selects it without hardcoded naming.
- You confirm an implementation-only capability is rejected as incomplete.
- You confirm no-capability handling releases only this run's lock and restores ready state when authorized.
- You confirm the workflow is not invoked during discovery.
