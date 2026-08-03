# 03 - Check Architecture

You compare the changed scope with the project's declared structural constraints.

## Input

- Use the changed files and applicable architecture sources for changed-scope mode, or the entire repository and all governing architecture sources for explicit global-report-only mode.

## Output

- Return a conformance result with macro and micro coverage and each violation's severity, evidence, constraint, and smallest corrective direction.

## Process

1. **Select mode.** You use changed-scope mode for implementation validation. You use global-report-only mode only when the user explicitly requests whole-project architecture conformance.
2. **Confirm applicability.** You run this facet when project instructions, architecture records, diagrams, module boundaries, or dependency rules define an expectation. You mark it not applicable otherwise.
3. **Load authority.** In changed-scope mode, you read the sources governing the changed files. In global-report-only mode, you read the project architecture memory, current tree contract, diagrams, decisions, and module rules required for complete coverage.
4. **Check macro boundaries.** You compare file placement, service ownership, public interfaces, and direct cross-domain dependencies with the declared structure.
5. **Check micro boundaries.** You inspect import direction, cycles, interface implementation, data ownership, and layer separation where the sources define them.
6. **Report evidence.** You cite a real path and the exact declared constraint for every violation, grouped into macro and micro results. On conformance, you state `no violations` for both groups.
7. **Repair narrowly.** In changed-scope mode only, you repair a violation when the fix is local, unambiguous, and inside the approved implementation scope. In global-report-only mode, you never edit, format, generate, or otherwise mutate project files.

## Stop conditions

- You stop before inventing an architecture rule that the project does not declare.
- You stop before a broad refactor or public-contract change.
- You return incomplete when the required authority source is referenced but unavailable.
- You never repair in global-report-only mode, even when a fix appears local.

## Test

- Every violation ties a changed file to a loaded project constraint.
- A clean result states that no violation was found in the inspected scope.
- An unresolved required violation prevents a pass verdict.
- Global-report-only mode covers the complete repository in continuable file batches and leaves repository state unchanged.
