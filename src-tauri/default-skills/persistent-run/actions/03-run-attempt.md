# 03 - Run an attempt

You execute one bounded hypothesis, preserve project state, and collect evidence without deciding completion yourself.

## Input

- Accept the reconciled tracking record, current unchecked step, and immutable action contract.
- Use prior attempt entries, the current failure evidence, and the remaining budgets.

## Output

- Return one attempt result with actions, observations, independent verification, and remaining budgets.
- Return one append-only log entry and any justified journey-map amendment.

## Process

1. You read the full tracking record and [the attempt log format](../references/attempt-log-format.md). You confirm the state is `pending` or `in-progress` and at least one attempt remains.
2. You choose the next unchecked step and state one falsifiable hypothesis about why its current acceptance check fails.
3. You compare the hypothesis, approach, and intended edits with prior failures for that step. You refuse an unchanged retry and describe the material difference.
4. You predict a measurable progress signal before changing anything. You include the expected verification command, file property, or observable result.
5. You increment the attempt number, set `status: in-progress`, and debit the attempt from the overall total before execution.
6. You use an isolated worker for the single attempt when delegation is available. You fill [the bounded worker prompt](../assets/attempt-worker-prompt.md) with only the relevant step, evidence, contract, and remaining boundaries. You execute directly under the same rules when delegation is unavailable.
7. You validate every input and path before use. You pass command arguments separately without a shell intermediary, bound output capture, redact sensitive data, and fail closed on errors.
8. You stay inside the action contract. You stop before any unlisted effect or conflict with user work. You perform an irreversible action, purchase, account or credential change, or external write only when that exact effect, target, occurrence limit, safeguards, and applicable cost ceiling appear in the contract from the user's original request.
9. You inspect the resulting diff or state and separate changes produced by this attempt from pre-existing changes. You do not discard, rewrite, or claim user work.
10. You verify the step independently with the predicted check. You treat an executor's report as evidence to inspect, never as proof.
11. You append exactly one log entry for the attempt, including the hypothesis change, actions, observed result, verification evidence, progress signal, state fingerprint, and remaining budgets.
12. You amend the journey map only when evidence invalidates an assumption. You record the rationale and keep prior history intact.

## Stop conditions

- You stop before execution when the proposed attempt repeats a failed hypothesis or exceeds any recorded boundary.
- You stop during execution when new work, unsafe drift, sensitive output, or an out-of-scope effect appears.
- You stop after the attempt when independent verification cannot be run or its result is inconclusive.
- You stop when the attempt exhausts the total even if more ideas remain.

## Test

- You confirm the attempt has one distinct hypothesis and one predeclared progress signal.
- You confirm all mutations and effects fit the immutable action contract.
- You confirm independent evidence matches the actual post-attempt state.
- You confirm the append-only log contains one and only one entry for the attempt.
