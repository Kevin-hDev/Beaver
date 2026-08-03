# 05 - Synthesize

You turn the reviewed evidence into one concise, prioritized verdict.

## Input

- Use candidate findings, coverage notes, the per-phase checklist, objective verification counts, and unplanned-change records from every applicable review axis.

## Output

- Return one schema-valid report with the per-phase checklist, objective percentage, unplanned changes, prioritized findings, final verdict, reviewed scope, axes not run, and residual uncertainty.

## Process

1. **Load the contract.** You read [review-validator.yml](../assets/review-validator.yml) and use its exact field names, section order, table columns, enumerations, sentinel text, and calculation rules. You reject extra sections or fields.
2. **Confirm axes and evidence.** You list the selected axes and mark every unselected or unavailable axis not run with a reason. You recheck every candidate against the changed line, surrounding behavior, authority source, and [evidence-rules.md](../references/evidence-rules.md).
3. **Reconcile functional records.** You verify that every authoritative phase and acceptance condition appears exactly once, recalculate the objective percentage, and reconcile every material diff change with one unplanned-change classification.
4. **Deduplicate.** You merge candidates with the same root cause and keep the highest supported severity.
5. **Prioritize.** You order findings from highest to lowest severity, then by qualitative confidence and user impact. You do not use the objective percentage as finding confidence.
6. **Write findings.** You give each finding a short imperative title, severity, qualitative confidence, narrow changed location, reachable scenario, impact, and corrective direction.
7. **Build and validate.** You fill [review-template.md](../assets/review-template.md) with verdict state `pending`, remove every placeholder, and validate the complete draft against the closed contract. You stop without a final verdict when validation fails.
8. **Assign verdict.** Only after the draft passes validation, you apply [review-rubric.md](../references/review-rubric.md), replace `pending` with the final verdict, and validate the final report again. You return incomplete when required evidence or full diff coverage is unavailable.
9. **Report cleanly.** You use the exact clean-report sentinel when the candidate set is empty. You still disclose axes not run and unverified behavior.
10. **Deliver.** You keep the result in the conversation by default. When the user requests a file, you validate the destination, stage the validated final report beside it, and replace only that report atomically.

## Stop conditions

- You remove any finding that lacks a changed causal location, reachable scenario, or concrete impact.
- You do not inflate severity to compensate for low confidence.
- You do not issue a final verdict when the closed report contract, checklist reconciliation, unplanned-change reconciliation, or percentage formula fails validation.
- You do not patch findings, stage files, commit, push, or post review comments externally.

## Test

- Every final finding meets the evidence contract and appears only once.
- The report matches the closed validator contract, and the final validation ran after verdict assignment.
- The objective percentage equals met eligible conditions divided by all eligible conditions and remains separate from qualitative confidence.
- The verdict matches the strictest supported finding and coverage state.
- A written report contains no placeholder and does not modify reviewed files.
