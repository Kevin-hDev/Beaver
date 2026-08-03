# 02 - Execute

You implement one bounded phase at a time and prove it before you continue.

## Input

- Use the prepared ordered scope, project rules, and acceptance checks from `01-prepare`.

## Output

- Return each completed phase with its focused tests passing, or a blocker or drift report with evidence.

## Process

1. **Select.** You take only the next incomplete phase or coherent task. You list its files, outcome, and checks before editing. In tracked-plan mode, you set that phase to `status: in-progress` as a runtime marker without a separate commit.
2. **Reuse.** You search again for an existing implementation at the point of change. You extend it instead of duplicating it.
3. **Test first when practical.** You add or update a focused test that fails for the missing behavior. When a pre-change test cannot express the behavior, you state why and add the test with the implementation.
4. **Edit minimally.** You apply the smallest cohesive change that satisfies the phase. You keep interface, business logic, and data access separated according to the project.
5. **Protect boundaries.** You validate untrusted inputs, bound external collections, handle errors closed, keep secrets out of code and logs, and avoid shell interpolation.
6. **Verify.** You run the focused test after every coherent edit. You repair a failure caused by the current change in batches of at most three attempts. After a failed batch, you preserve the attempts and evidence, revise the next batch's hypotheses, and continue until the check passes or a blocker in [implementation-guardrails.md](../references/implementation-guardrails.md) applies.
7. **Inspect.** You review the phase diff for accidental files, dead code, duplicated logic, leaked sensitive data, and unrelated formatting.
8. **Gate.** You compare the result with the phase acceptance checks. You continue only when they pass.
9. **Record tracked phase.** In tracked-plan mode, you set the phase to `status: done`, stage only its code, tests, and phase status, create one atomic phase commit, and verify that no phase-owned edit remains outside that commit. You never include unrelated user changes.
10. **Repeat.** You move to the next phase only after the current phase is clean and verified. When more than 20 phases remain, you begin the next continuable phase batch without losing order or evidence.

## Stop conditions

- You stop and report `replan needed` when repository evidence contradicts the approved plan or the required scope changes.
- You stop and report `blocked` when only a human, secret, unavailable service, hardware action, or external decision can continue the work.
- You stop only when a real blocker remains after a repair batch, not merely because one three-attempt batch ended.
- You do not alter unrelated user work or hide a failing check.
- In tracked-plan mode, a human-only blocker sets the plan to `status: blocked`, commits only that status with any required phase evidence, and stops before later phases.

## Test

- Every changed behavior has a focused passing test unless the project makes automated coverage impossible and the user accepts a named alternative.
- Every completed phase satisfies its observable acceptance checks.
- The diff contains no unrelated file or silent scope expansion.
- In tracked-plan mode, every completed phase has exactly one atomic code-plus-status commit and no separate `in-progress` commit.
