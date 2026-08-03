# 04 - Report

You summarize the executed tests or journey without overstating coverage or hiding failures.

## Input

- Use the selected behaviors or journey steps.
- Use actual commands, outputs, evidence, changed test files, and blockers.

## Output

- Return a concise status for every selected behavior or meaningful journey step.
- Return a final result of `passed`, `failed`, `blocked`, or `no-change`.

## Process

1. **Reconcile scope.** You account for every selected behavior or step exactly once across all continuable batches, including approved, pending, already-covered, blocked, and invalidated items.
2. **Separate results.** You distinguish new test results, pre-existing failures, confirmed product defects, and checks not run.
3. **Name evidence.** You provide the focused command or journey evidence that supports each result.
4. **State gaps.** You list every pending, deferred, or blocked behavior with one concrete reason and no invented assurance.
5. **Return verdict.** You use `passed` only when every selected check passed, `failed` for an observed contract violation, `blocked` when proof is unavailable, and `no-change` when coverage was already sufficient.

## Stop conditions

- You never report a wider suite, browser step, platform, or environment you did not actually exercise.
- You never turn a pre-existing red suite into a pass by ignoring it.
- You never expose raw secrets, private test data, internal stack traces, or unnecessary local paths.

## Test

- The report maps every selected item to actual evidence and one status.
- The verdict follows the worst unresolved applicable result.
