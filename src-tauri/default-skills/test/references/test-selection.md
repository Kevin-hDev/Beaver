# Test Selection

Read this reference while you assess coverage or choose the lowest useful test level.

## Prefer observable behavior

- You assert public output, state transition, persisted effect, emitted event, or stable error contract.
- You avoid private function calls, internal call counts, incidental markup, and snapshots that change without behavioral impact.
- You cover a boundary only when it can change the outcome or protect a meaningful contract.

## Choose the test level

| Level | Choose it when | Avoid it when |
| --- | --- | --- |
| Unit | One isolated rule has a stable public contract | Framework wiring or serialization is part of the behavior |
| Integration | Multiple local components, storage, routing, or serialization form the contract | The test would require a real external or production service |
| Contract | A schema or protocol boundary must remain compatible | The assertion merely repeats implementation types |
| End to end | Only the assembled system proves the user-visible journey | A lower level proves the same risk faster and more deterministically |

## Prioritize gaps

1. You prioritize security and access boundaries.
2. You prioritize core user outcomes and irreversible state transitions.
3. You prioritize regressions in recently changed or historically unstable behavior.
4. You prioritize error paths that must fail closed or avoid data loss.
5. You deprioritize duplicate permutations and implementation-only branches.

## Quality checks

- You keep each test deterministic and independent of execution order.
- You isolate time, randomness, filesystem, network, and shared state using the project's established helpers.
- You clean up created state even when the assertion fails without masking the original failure.
- You use bounded polling with an explicit success condition instead of an arbitrary fixed sleep.
- You keep one clear behavioral reason for each test to fail.
