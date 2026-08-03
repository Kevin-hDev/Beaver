# 05 - Finalize

You combine the facet results into one accurate validation verdict.

## Input

- Use the validation matrix and final results from every applicable facet.

## Output

- Return `pass`, `fail`, or `incomplete` with checks run, evidence, repairs, skipped checks, and remaining risk.

## Process

1. **Reconcile.** You confirm that every required matrix entry has a final result and that every optional or skipped entry has a reason.
2. **Assign verdict.** You return `pass` only when every required gate passed its final clean sweep. You return `fail` for a confirmed violation or failed required gate. You return `incomplete` when required evidence could not be obtained.
3. **Summarize evidence.** You list exact commands or journeys actually run, their result, the relevant bounded output, and the UI hypothesis journal reference when that facet repaired behavior. You sanitize secrets and internal sensitive values.
4. **List repairs.** You list changed files and the check that proved each repair.
5. **Expose gaps.** You list skipped checks, pre-existing failures with evidence, and residual risks without minimizing them.
6. **Deliver.** You keep the report in the conversation. When the user requests a file, you copy [validation-report-template.md](../assets/validation-report-template.md), validate the destination, remove placeholders, and write atomically.

## Stop conditions

- You do not report `pass` when a required check failed, was skipped, timed out, or lacks evidence.
- You do not hide pre-existing failures or attribute them to the current work without proof.
- You do not perform new repairs after you begin the final report; you return to the relevant facet first.

## Test

- The verdict follows the strictest required facet result.
- Every claimed pass has tool or runtime evidence.
- A written report contains no placeholder or sensitive value.
