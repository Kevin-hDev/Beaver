# 03 - Complete

You run a final clean verification and report exactly what the implementation achieved.

## Input

- Use the completed requested phases, their acceptance checks, and the prepared validation gates.

## Output

- Return a pass or fail result, changed behavior, files changed, tests run, and remaining risks or blockers.

## Process

1. **Confirm scope.** You verify that every requested phase is complete and that no unresolved drift remains.
2. **Run gates.** You run the required repository-defined tests, type checks, lint checks, and builds that apply to the changed scope. You pass system arguments separately, avoid a shell intermediary, never run development mode, and never use a formatting command as validation.
3. **Repair carefully.** You fix only failures caused by the current implementation. You work in batches of at most three attempts per gate, preserve evidence across batches, and continue until each required gate passes or a real blocker remains. You rerun affected focused tests after every edit.
4. **Sweep.** You rerun the complete selected gate set in one final clean pass.
5. **Inspect state.** You inspect the final diff and version-control status. You confirm that unrelated edits remain unchanged and no sensitive data appears.
6. **Finalize tracked plan.** In tracked-plan mode, you require every phase to read `status: done`, set the plan to `status: implemented`, commit that final status as its own lifecycle commit, and confirm the branch contains the ordered phase commits plus this final commit.
7. **Report.** You list the execution mode, checks actually run, their result, changed behavior, files, lifecycle commits when applicable, checks not run with reasons, and remaining risk. You do not claim completion when a required gate failed or was skipped.

## Stop conditions

- You return a failed or incomplete result when any required gate fails, cannot run, or exceeds its bounded repair attempts.
- You stop before broad formatting, generated-file rewrites, dependency changes, or external actions outside the approved scope.
- In workspace mode, you do not commit, push, or create a pull request. In tracked-plan mode, you create only the lifecycle commits defined by the approved scope and never push or open a pull request without explicit publication scope.

## Test

- The final clean sweep passes every required selected gate before you report success.
- The summary matches the actual diff and command results.
- The working tree contains no new unrelated changes from the implementation.
- In tracked-plan mode, the plan is committed as `implemented`, every phase is committed as `done`, and phase boundaries contain no dangling edits.
