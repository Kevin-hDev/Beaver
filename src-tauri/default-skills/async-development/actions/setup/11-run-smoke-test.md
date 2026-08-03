# 11 - Run Smoke Test

You exercise the configured pipeline with an isolated disposable ticket and bounded observation.

## Input

- Use the confirmed configuration, effect contract, detection report, and verified setup artifacts.
- Accept separate explicit authority for smoke creation, trigger effects, and cleanup effects.

## Output

- Return the smoke ticket identity, observed run identity, optional change-request identity, bounded outcome, cleanup status, and verification evidence.
- Return `skipped` without external mutation when smoke authority is denied.

## Process

1. You require explicit smoke-test authority and show every expected external effect before creation.
2. You create a new self-contained disposable ticket with a stable smoke marker, a unique idempotency key, and a minimal project-local change. You never reuse a backlog item or reference another ticket for closure.
3. You apply the configured ready trigger only when that state mutation is separately authorized and verified.
4. You observe the exact smoke run through the integration or invoke the local runner once. You use a configurable finite deadline and polling interval.
5. You scope every observation to the smoke ticket and idempotency key. You accept only an observed terminal result, verified change request, verified blocked state, or timeout.
6. You ensure the smoke workflow never merges into or writes directly to the default branch. You stop immediately if the default branch revision changes during the test.
7. You require separate cleanup authority before closing the smoke change request, deleting its dedicated branch, closing the smoke ticket, or removing generated smoke artifacts.
8. You clean up only objects created by this smoke run and verify each object independently. You never revert the default branch as an automated cleanup strategy.
9. You return exact remaining identifiers and safe resumable cleanup steps when cleanup is denied or incomplete.

## Stop conditions

- You stop without mutation when smoke creation or trigger authority is absent.
- You stop immediately on default-branch drift, ambiguous object identity, timeout, or an unverifiable state change.
- You stop cleanup on the first failed critical operation and preserve the remaining object inventory.

## Test

- You confirm a declined smoke test creates no ticket, label change, run, branch, or change request.
- You confirm an authorized smoke test touches only objects carrying its unique marker.
- You confirm the observation deadline is finite and scoped to one run.
- You confirm cleanup never reverts the default branch and verifies every removed or closed object.
