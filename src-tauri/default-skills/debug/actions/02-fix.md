# 02 - Fix

You encode the defect as a failing regression test and apply one minimal fix at the confirmed source.

## Input

- Use the reliable reproduction and localized cause from `01-reproduce`.

## Output

- Return one regression test that failed for the expected reason and one bounded production fix.
- Return the failing-test and fix commits when full-delivery mode was explicitly selected.

## Process

1. **Confirm cause.** You restate the evidence connecting the source-level cause to the reproduced symptom. You do not continue on a plausible guess.
2. **Select test.** You read [regression-test.md](../references/regression-test.md), inspect existing coverage, and choose the smallest stable test surface.
3. **Prove failure.** You add or update one focused regression test and run it before the fix. You confirm that it fails because of the reported defect, not setup or an unrelated error. In full-delivery mode, you commit this failing test alone and link the verified issue identifier.
4. **Fix once.** You change the earliest correct source of the defect with the smallest cohesive patch. You reuse existing validation and abstractions.
5. **Protect boundaries.** You validate untrusted input, bound external collections, fail closed, keep secrets out of code and logs, and preserve error handling.
6. **Run focused test.** You rerun the regression test and record its actual result.
7. **Record the fix.** In full-delivery mode, you commit the minimal production fix with its passing focused evidence and link the same issue identifier.
8. **Inspect diff.** You remove dead code and temporary diagnostics created by this fix and exclude unrelated formatting or cleanup.

## Stop conditions

- You stop before production edits when the test does not fail for the expected reason.
- You stop when the required fix expands into a new public contract, dependency, migration, or architecture decision.
- You do not weaken existing assertions or hide a failure.

## Test

- The same focused test fails before the fix and passes after it.
- The production diff addresses only the confirmed cause.
- No temporary diagnostic or sensitive value remains.
- Full-delivery mode contains one failing-test commit followed by one minimal-fix commit, both linked to the issue.
