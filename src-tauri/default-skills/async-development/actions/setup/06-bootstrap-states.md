# 06 - Bootstrap States

You create missing lifecycle states idempotently and fail closed on any unverified transition.

## Input

- Use the confirmed configuration, effect contract, and tracker adapter capability record.
- Use the configured ready, review, working, awaiting-review, and blocked state definitions.

## Output

- Return each state as `created`, `already-present`, `denied`, or `failed`, with independent verification evidence.
- Return a complete or blocked setup status.

## Process

1. You require explicit authority to create states on the exact tracker target. You return `denied` without mutation otherwise.
2. You list existing states through a bounded, paginated adapter query and normalize names according to documented provider rules.
3. You compare every configured state exactly. You preserve an existing state's color or description unless changing it was separately authorized.
4. You create only missing states through documented adapter operations and validated parameters.
5. You read the state list again after every creation and require an exact identity match.
6. You stop immediately on the first creation or verification failure. You record the known completed subset and a resumable next state.
7. You never continue after a critical failure and never report five available states unless all five are independently observed.

## Stop conditions

- You stop before mutation when the adapter cannot list, create, or verify lifecycle states.
- You stop when the configured names collide after provider normalization.
- You stop on the first failed or unverifiable state creation.

## Test

- You confirm a first authorized run creates every missing configured state and verifies it independently.
- You confirm a second run creates nothing and returns every state as `already-present`.
- You force one creation failure and confirm later states are not attempted.
- You confirm a denied contract performs no mutation.
