# 03 - Review Functional

You trace the change against its intended behavior and acceptance conditions.

## Input

- Use the plan, specification, issue, pull-request description, or explicit user request resolved during collection.

## Output

- Return a per-phase acceptance checklist, its objective verification counts and percentage, and explicit unplanned-change records with evidence.

## Process

1. **Validate authority.** You identify the source that defines the intended behavior and preserve its exact hard constraints. When none is available, you ask once for acceptance criteria and mark the axis not run only if they remain unavailable.
2. **Enumerate phases and conditions.** You preserve the source phase order and assign every distinct condition a stable `P<phase>-AC<condition>` identifier. You extract at most 100 conditions per numbered batch and continue later batches without dropping entries. When the authority has no phases, you use one `Change set` phase without inventing implementation phases.
3. **Trace evidence.** You connect each condition to changed code, tests, configuration, migrations, or documentation that implements it.
4. **Classify.** You assign exactly one of `met`, `unmet`, `partial`, `not-applicable`, or `unverified`. You mark a condition met only when direct evidence satisfies it and unverified when required runtime proof was not run.
5. **Build the checklist.** You emit one checklist row per phase and condition, including identifier, exact condition, status, and evidence or missing-evidence note.
6. **Calculate objective verification.** You exclude `not-applicable`, set the denominator to all remaining checklist rows, and set the numerator to `met` rows only. You report the nearest whole percentage as `met / eligible × 100`; when the denominator is zero, you report `N/A`, never `100%`.
7. **Identify gaps.** You create a candidate finding for every unmet or partial required condition and explain the missing behavior.
8. **Track unplanned changes.** You inventory at most 100 material user-visible, public-contract, data, permission, dependency, configuration, or migration changes per numbered batch and continue until all are classified. You classify each as `planned`, `unplanned`, or `unclear`, link supporting authority when present, and create a candidate finding for every impactful unplanned change.
9. **Separate confidence.** You use qualitative confidence only for evidence quality. You never substitute it for, average it with, or infer it from the objective acceptance-verification percentage.
10. **Avoid execution claims.** You do not treat static inspection as proof that a runtime journey passed.

## Stop conditions

- You mark this axis not run when no authoritative intended behavior is available and the user did not supply it.
- You stop before inventing acceptance conditions or changing the plan.
- You do not patch an unmet condition.

## Test

- Every condition has one status and supporting evidence or a precise missing-evidence note.
- Every authoritative phase has a checklist row, every condition identifier appears exactly once, and the objective percentage matches the stated formula.
- Every unmet required condition maps to a candidate finding.
- Every material change has an explicit planned, unplanned, or unclear classification.
- No static-only check is described as a successful runtime validation.
