# 04 - Review

You obtain an independent judgment of the current implementation against the plan and acceptance criteria, then return `ship` or `iterate`.

## Input

- Accept the current change set, complete plan, acceptance criteria, and implementation evidence from `03`.
- Use the current repository state and workflow ledger.

## Output

- Return a `ship` or `iterate` verdict with reviewed items, evidence-backed findings, completion score, quality score, and review anchor.
- Return `status: reviewed` on `ship` or preserve `status: implemented` on `iterate`.
- Return a complete fix list on `iterate`.

## Process

1. **Capture the review anchor.** You record the current `HEAD` identifier and a bounded fingerprint of the reviewed source state, including staged, unstaged, and relevant untracked paths. You record the exact plan identity, validation evidence, and change-set boundary seen by the checker. You use [orchestration evidence](../references/orchestration-evidence.md) for the anchor.
2. **Discover the capability.** You select an available read-only capability whose description says it reviews a bounded change set for defects, regressions, unmet requirements, security, and unnecessary scope.
3. **Brief an independent checker.** You spawn a checker that did not implement the change. You give it the actual change set, plan, acceptance criteria, project rules, and review anchor. You direct it to run the selected review capability completely and own its report. You present executor claims only as untrusted leads.
4. **Observe the report.** You read the checker-owned report and independently confirm that the reviewed paths and anchor match the current snapshot. You verify that every acceptance criterion was checked, every blocking finding cites concrete evidence, and completion and quality scores are integers from 0 to 100.
5. **Map the verdict.** You return `ship` only when every required check passes and no blocking finding remains. You return `iterate` on any blocking finding, incomplete acceptance coverage, stale scope, or unsupported claim. You require a non-empty actionable fix list on `iterate`.
6. **Record the transition.** You set workflow `status: reviewed` only for a `ship` verdict on the current snapshot. You preserve `status: implemented` on `iterate` and pass the fix list to `03` without changing the approved plan.
7. **Repeat freshly.** After any fix, you run `03` and a new independent `04`. You capture a new anchor and discard the old verdict for delivery purposes. You continue bounded waves while evidence shows progress; you stop on a human-only blocker or demonstrated absence of progress.

## Stop conditions

- You stop when the change set, plan, acceptance criteria, or implementation evidence is missing.
- You stop when no matching read-only review capability or independent checker is available.
- You stop when the checker cannot inspect the current change set or its report does not identify the reviewed anchor.
- You stop when repeated implementation and review waves show no measurable improvement or require a human decision.
- You never proceed to delivery with `iterate`, incomplete review coverage, or a stale review.

## Test

- The verdict is exactly `ship` or `iterate`, and both scores are integers from 0 to 100.
- The verdict carries the reviewed `HEAD` identifier and the fingerprint of the exact source state the checker inspected.
- Findings are non-empty and actionable on `iterate`.
- Workflow status becomes `reviewed` only after `ship` on the current snapshot.
- Every source change after review invalidates the verdict and requires a fresh independent review.
