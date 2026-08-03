# 02 - Refine and Stage

You refine each todo and construct safe, continuable launch waves.

## Input

- Accept the validated task ledger, dependency graph, current project state, and available executor capabilities.

## Output

- Return a refined contract for every task and an ordered wave plan.
- Return one structured executor prompt per ready task, with no shared write ownership inside a wave.

## Process

1. **Refresh readiness.** You reconcile the ledger with the current files and mark a task ready only when every predecessor is verified.
2. **Refine every todo.** You use a non-interactive refinement capability when one is available through runtime discovery. Otherwise, you restate the task precisely and resolve obvious ambiguity from project evidence.
3. **Preserve unresolved boundaries.** You mark a task blocked instead of guessing when refinement would change product intent, authority, or a material contract. You do not let an executor ask the user; you keep user coordination at the orchestrator level.
4. **Define the contract.** You specify the objective, relevant context, allowed read and write scope, forbidden effects, required process, validation, output format, success criteria, and reflection gate.
5. **Create waves.** You select up to six ready tasks with disjoint write scopes and compatible resource effects. You postpone all other ready tasks to later waves without treating postponement as cancellation.
6. **Serialize conflicts.** You add ordering edges for shared files, formatters that rewrite broad areas, generated indexes, schemas, migrations, locks, ports, accounts, or any newly discovered collision.
7. **Instantiate prompts.** You copy the structure from [executor-prompt-template.md](../assets/executor-prompt-template.md) and tailor it to exactly one task. You mandate refinement first, implementation second, verification third, and a one-line summary last.
8. **Recheck the wave.** You compare every prompt with the ledger and [executor-contract.md](../references/executor-contract.md) before launch.

## Stop conditions

- You do not stage a task until its predecessors are verified.
- You do not put two tasks with overlapping possible writes or effects in the same wave.
- You do not launch more than six executors concurrently in the active session.
- You do not invent missing authority or material requirements during refinement.

## Test

- You confirm that every ready task has one dedicated prompt and that each prompt begins with refinement.
- You confirm that each prompt names exclusive write ownership and explicit forbidden effects.
- You confirm that one wave contains no more than six tasks and that all remaining tasks remain queued with stable IDs.
- You confirm that no dependency, conflict, or blocked task was staged early.
