# 02 - Explore

You inspect the repository to map the smallest credible implementation surface.

## Input

- Use the confirmed scope from `01-gather`.
- Use the repository root resolved from the current workspace or supplied explicitly by the user.

## Output

- Return applicable project rules, reusable existing code, an impact map, validation commands, dependencies, risks, and open uncertainties.

## Process

1. **Validate.** You resolve the repository root and keep every inspected path inside it.
2. **Read rules.** You load the project instructions that apply to the likely files before you project changes.
3. **Trace.** You follow the current behavior across interface, business logic, data access, tests, and configuration only as far as the requested scope requires.
4. **Reuse.** You search for existing components, functions, patterns, constants, and tests before you propose additions.
5. **Map.** You process at most 100 projected files per batch. You list files likely to modify, create, or delete with one evidence-based reason each and continue additional batches until coverage is complete. You mark uncertain paths as candidates instead of facts.
6. **Verify.** You identify the repository-defined commands that can verify the work. You consult official documentation only when repository evidence cannot settle a changing technical fact.
7. **Assess.** You record dependencies, migration or compatibility concerns, sensitive boundaries, and risks.

## Stop conditions

- You stop when the repository root or required project rules cannot be resolved.
- You stop when feasibility depends on a missing decision or inaccessible source.
- You do not modify files, install packages, start services, or run destructive commands.

## Test

- Every projected file has a reason tied to inspected evidence.
- The output names reusable existing code before any proposed new equivalent.
- Every validation command comes from project instructions or detected configuration.
