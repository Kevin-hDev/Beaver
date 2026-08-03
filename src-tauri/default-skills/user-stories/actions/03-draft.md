# 03 - Draft

You turn each vertical slice into an INVEST story with observable acceptance and completion conditions.

## Input

- Accept candidate stories and approved product constraints.

## Output

- Return each complete story with acceptance, completion, assumptions, and dependencies.

## Process

1. **Write need.** You name the user, desired capability, and value without freezing a technical solution.
2. **Write criteria.** You cover the nominal outcome and relevant empty, error, permission, limit, accessibility, or recovery behavior.
3. **Write functional completion.** You state two to five user-observable conditions that prove the outcome.
4. **Check INVEST.** You test all six criteria and return oversized or dependent stories to slicing.
5. **Expose uncertainty.** You keep unresolved acceptance decisions visible and mark the story not ready.

## Stop conditions

- You do not invent product rules, numeric limits, roles, or error behavior.
- You do not include implementation steps, file names, frameworks, database tasks, review, deployment, or coverage in functional completion.

## Test

- Every story is INVEST-compliant or explicitly not ready with its blocking question.
- Acceptance criteria are observable and include every relevant failure or boundary already known.
