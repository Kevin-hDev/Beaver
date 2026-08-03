# 01 - Decompose and Graph

You turn the full request into a complete dependency and ownership graph without implementing it.

## Input

- Accept the user's multi-part requirement and the current project state.
- Accept optional scopes, validation commands, deadlines, and explicitly authorized effects.

## Output

- Return a task ledger with a stable ID, category, task, requirements covered, dependencies, read scope, exclusive write scope, expected result, and verification for every task.
- Return a directed acyclic graph or mark the exact cycle that prevents safe scheduling.

## Process

1. **Read the request.** You ask for the requirement only when it is empty. You identify requested deliverables, constraints, exclusions, proof, and effects.
2. **Inspect the current state.** You locate relevant files, project rules, existing implementations, tests, and user changes before you divide ownership.
3. **Categorize the work.** You split the request into the smallest useful tasks that each produce a coherent result. You keep tightly coupled edits in one task.
4. **Map coverage.** You assign every requirement to one primary task and record cross-task integration checks separately. You remove duplicates without losing intent.
5. **Build dependencies.** You add an edge whenever a task needs another task's output, schema, decision, generated file, or verified state. You distinguish read-only relationships from ordering constraints.
6. **Assign ownership.** You give each task exclusive write paths or resources. You serialize tasks when their possible effects overlap, even if their descriptions sound independent.
7. **Validate the graph.** You detect cycles, hidden shared state, generated-file collisions, and tasks whose boundaries are too vague to delegate safely.
8. **Scale without dropping work.** You record large task sets in continuable ledger sections and load bounded windows while scheduling. You preserve stable IDs and all unprocessed rows across waves.

## Stop conditions

- You stop and ask for the requirement when no prompt was supplied.
- You do not launch a task whose scope, expected result, or verification is missing.
- You mark only the affected task `blocked` when material ambiguity cannot be resolved from available evidence, and you continue graphing unrelated work.
- You do not force a cyclic or overlapping graph into parallel execution.

## Test

- You confirm that every requested deliverable and constraint maps to the ledger.
- You confirm that every dependency edge has a concrete reason and that the graph is acyclic before scheduling.
- You confirm that concurrently eligible tasks have disjoint write scopes and compatible effects.
- You confirm that every task has a deterministic or evidence-based verification method.
