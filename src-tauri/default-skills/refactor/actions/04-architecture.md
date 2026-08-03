# 04 - Architecture

You restore documented boundaries and dependency direction through small independently verified changes.

## Input

- Accept an optional validated file, directory, module, or glob scope, defaulting to the current codebase.
- Accept optional pasted or readable architecture findings from an audit report.

## Output

- Return every applied atomic change with its file, severity, boundary purpose, concrete edits, and per-step verification.
- Return every structural move unsafe to perform atomically in a deferred list marked `needs a plan`, with dependencies, risk, and reason.

## Process

1. **Resolve scope.** You validate the selection, read applicable project instructions, and protect unrelated edits.
2. **Discover boundaries.** You read the project's established architecture evidence using [architecture-boundaries.md](../references/architecture-boundaries.md): architecture decisions, diagrams, module manifests, package boundaries, dependency rules, tests, and representative imports. You do not invent a target architecture from personal preference.
3. **Build the fix list.** You use current architecture audit findings when supplied and skip broad discovery; otherwise you identify wrong-direction dependencies, broken isolation, missing or bypassed layers, god modules, cyclic coupling, and code placed across a documented boundary. You rate only supported findings.
4. **Establish behavior and graph.** You run existing focused tests and type checks, record representative public behavior, and capture the relevant pre-change import or dependency relationships.
5. **Triage.** You separate changes safe to make atomically from broad coordinated moves involving public contracts, data migration, multiple ownership domains, or an unverifiable intermediate state. You place every broad move in the deferred `needs a plan` list before editing.
6. **Apply atomically.** You make one safe step at a time: extract or restore domain, infrastructure, and presentation layers; introduce an interface or inversion point to correct dependency direction; split god modules along natural responsibilities; or move code and adjust exports and internal references to enforce an established boundary.
7. **Verify each step.** You run focused tests and type checks after every step, compare public behavior with the baseline, and inspect the relevant import graph for new or remaining violations. You keep a step only when its checks pass independently.
8. **Close.** You run the complete changed-scope checks, map every claim to a concrete diff, and report the deferred list. You recommend planning deferred moves before any related code movement.

## Stop conditions

- You defer a move rather than begin an unbounded rewrite when it cannot be completed and verified atomically.
- You stop before violating an established boundary merely to shorten the change.
- You report `baseline unavailable` when behavior or dependency direction cannot be established and do not claim preservation.
- You report `incomplete` when any required per-step test, type check, behavior comparison, or boundary check fails.

## Test

- Existing focused tests and type checks pass after every retained step.
- Public inputs, outputs, errors, ordering, and side effects match the baseline where behavior must be preserved.
- The final import or dependency graph contains no new boundary violation and resolves every claimed applied finding.
- Every non-atomic structural move appears in the deferred `needs a plan` list.
