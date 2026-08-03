# 03 - Execute Waves

You run ready tasks in parallel waves, inspect their real effects, and reconcile the shared state.

## Input

- Accept one validated wave, its executor prompts, the task ledger, and the pre-wave project state.

## Output

- Return one reconciled ledger update per launched task with executor identity, launch state, observed changes, verification evidence, and status.
- Return the next set of newly ready, failed, or blocked tasks.

## Process

1. **Capture the baseline.** You record the relevant pre-wave state and existing user changes so you can distinguish executor effects without discarding unrelated work.
2. **Launch the wave.** You launch one dedicated executor per staged task and dispatch the entire wave as one concurrent launch operation when the available orchestration interface supports it. You never start the wave sequentially merely for convenience, and you never exceed six active executors.
3. **Enforce task order.** You require each executor to refine, implement, verify, and summarize in that order. You require it to stop before touching any path or effect outside its contract.
4. **Collect all results.** You wait for every launched executor to reach a terminal response. You treat missing, interrupted, or vague responses as incomplete rather than successful.
5. **Inspect reality.** You inspect the actual changed files, diffs, command results, tests, and artifacts for each task. You compare them with the baseline and task contract instead of trusting the one-line summary.
6. **Reconcile interactions.** You detect cross-task regressions, overwritten work, stale assumptions, broad formatter changes, generated output changes, and new dependencies. You preserve user work and the valid portions of completed tasks.
7. **Classify each task.** You mark a task `verified`, `failed`, `blocked`, or `incomplete` with direct evidence. You mark dependents ready only after all predecessors are `verified`.
8. **Continue in waves.** You return to refinement and staging for the next ready tasks. You keep processing later waves until every ledger task reaches a terminal status. You apply the six-executor ceiling only to simultaneous execution.

## Stop conditions

- You stop a task when it attempts an unowned write, unauthorized effect, destructive action, or incompatible shared-resource change.
- You do not mark a task verified from its executor's assertion alone.
- You do not launch a dependent task after a predecessor fails, blocks, or remains incomplete.
- You stop starting new work when safe reconciliation is impossible, but you retain all evidence and completed rows.

## Test

- You confirm that launched executors never exceeded six concurrently and that every launched task had exactly one owner.
- You confirm that the observed changes stay within each task's exclusive contract or are marked as violations.
- You confirm that every `verified` status cites direct task-specific evidence.
- You confirm that all unlaunched tasks remain represented and eligible for a later wave when their dependencies permit it.
