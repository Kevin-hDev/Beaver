# Executor Contract

You use one contract for one task and one dedicated executor.

## Required prompt sections

You include these six sections in every executor prompt:

1. You provide `<context>` with only relevant project state, task dependencies, rules, and owned paths.
2. You provide `<task>` with one measurable deliverable.
3. You provide `<constraints>` with allowed writes, forbidden effects, preservation rules, and validation boundaries.
4. You provide `<output_format>` with the exact evidence and one-line summary required.
5. You provide `<success_criteria>` with observable completion checks.
6. You provide `<reflection>` that requires a final scope, test, and side-effect audit.

## Required executor order

You mandate this order inside the contract:

1. You refine the todo non-interactively with an available runtime refinement capability, or you restate it precisely from supplied evidence.
2. You implement only the refined todo within the exclusive scope.
3. You run the requested focused validation and capture exact evidence.
4. You inspect your own changes for scope drift, unrelated edits, and missing requirements.
5. You return structured evidence and a one-line output summary.

You forbid the executor from asking the user directly. You require it to return `blocked` with the exact missing decision when material intent cannot be derived safely.

## Evidence contract

You request changed paths, commands or inspections performed, observed outcomes, remaining risks, and status. You treat all of these as claims until the orchestrator independently inspects them.
