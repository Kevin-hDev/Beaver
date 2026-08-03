# 02 - Plan

You turn the normalized contract into the mandatory implementation plan. You never substitute the request, specification, or a task list for this plan.

## Input

- Accept the specification location when specification creation produced an artifact.
- Accept the objective, acceptance criteria, source map, raw request, and unresolved decisions returned by `01`.
- Use the working scope, applicable project rules, and workflow ledger.

## Output

- Return the plan location, ordered phases, acceptance mapping, decisions, unresolved blockers, and validation result.
- Return workflow `status: pending` with the plan identity recorded in the ledger.

## Process

1. **Confirm the contract.** You verify that the objective is explicit and at least one acceptance criterion is present. You read the specification when its location is not `null`; otherwise you use the exact skipped fields and raw request.
2. **Discover the capability.** You select an available capability whose description says it creates a repository-grounded implementation plan with phases, file impact, risks, and observable checks. You do not select an implementation worker or allow a worker to author the plan.
3. **Run complete planning.** You invoke that capability in the current orchestration context and let it own source gathering, project exploration, phase design, plan validation, and any interaction its contract requires. You keep its full artifact in the conversation unless the user selected a validated project-local destination or the project established one.
4. **Capture decisions.** You read the complete returned plan and record its location, identity, phases, acceptance mapping, decisions, unresolved blockers, and stated validation evidence. You never inline a raw ticket or specification as the plan body.
5. **Verify the plan.** You confirm that the plan objective matches the specification or exact skipped objective, every acceptance criterion maps to at least one task and observable check, phase order is executable, and the complete artifact reports `status: pending`.
6. **Record the transition.** You record `status: pending` and the verified plan evidence in the workflow ledger. You pass the plan itself, not a summary, to implementation.

## Stop conditions

- You stop when the objective, acceptance criteria, working scope, or applicable project rules are unavailable.
- You stop when an unresolved decision materially changes scope, architecture, data handling, or delivery.
- You stop when no matching planning capability is available.
- You stop when the returned artifact is missing, unvalidated, substitutes raw source text for a plan, or fails to cover an acceptance criterion.

## Test

- The complete plan is observable at its recorded location and reports `status: pending`.
- The plan objective matches the specification objective or the exact skipped objective.
- Every acceptance criterion maps to implementation work and an observable check.
- The ledger records the plan identity and evidence rather than only a prose summary.
