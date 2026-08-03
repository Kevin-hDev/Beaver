# 03 - Verify

You prove that the defect is resolved without introducing a regression.

## Input

- Use the original reproduction, regression test, fix diff, and applicable project checks.

## Output

- Return a pass or fail verdict with the checks actually run, their results, changed files, remaining risk, and the requested pull-request result when full-delivery mode applies.

## Process

1. **Run regression test.** You run the focused test from `02-fix` and require a clean pass.
2. **Repeat original trigger.** You exercise the original reproduction and confirm the actual behavior now matches the expected behavior.
3. **Run affected checks.** You run the repository-defined tests, type or compile checks, lint, and builds required by the changed scope.
4. **Inspect final state.** You review the diff for scope creep, temporary instrumentation, sensitive values, dead code, and unrelated changes.
5. **Assign verdict.** You report success only when the regression test, original trigger, and every required affected check pass.
6. **Return to diagnosis.** When the fix is disproved or reveals an unclear wider cause, you preserve sanitized evidence and continue with `04-investigate-cause` or `05-reflect-issue` without trying another production fix.
7. **Complete explicit delivery.** In full-delivery mode only after every required check passes, you confirm the two commits and scope, push the dedicated branch, and open one pull request that links the issue with `Fixes #<issue-id>`. You report the verified URL, branch, issue identifier, and provider state.

## Stop conditions

- You return fail when a required check fails, times out, or cannot run.
- You do not delete or weaken a failing test.
- In local mode, you do not commit, push, or publish. In full-delivery mode, you perform only the explicitly requested issue, branch, two commits, push, and pull-request lifecycle.

## Test

- A pass verdict has evidence from both the regression test and original trigger.
- Every required affected check has a reported real result.
- A disproved fix resumes internal diagnosis, not another speculative patch or external delegation.
- Full-delivery success includes a verified pull-request URL, dedicated branch, linked issue, and passing full suite.
