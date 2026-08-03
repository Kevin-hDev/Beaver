# 01 - Prepare

You establish a safe, repository-grounded baseline before you modify code.

## Input

- Accept an approved plan path, inline plan, or precise bounded task.
- Use the repository root resolved from the current workspace or supplied explicitly by the user.

## Output

- Return the execution mode, ordered phase or task scope, applicable rules, existing user changes, reusable code, validation gates, and Git baseline when tracked-plan mode applies.

## Process

1. **Validate.** You validate traversal, readability, source structure, and phase ordering. You inspect at most 256 KiB from a plan file or 100,000 inline characters per input batch and continue later batches until the complete plan is resolved. You process oversized phase and task lists through the continuable execution batches below instead of rejecting them.
2. **Select lifecycle.** You read [execution-lifecycle.md](../references/execution-lifecycle.md). You select workspace mode unless the plan, project rules, or user explicitly requires the tracked plan-status and atomic Git outcome.
3. **Inspect state.** You inspect the version-control status. You identify unrelated edits and files that overlap the requested scope. In tracked-plan mode, you also resolve the current and default branches and require a Git repository.
4. **Read rules.** You load every project instruction that applies to the target files.
5. **Trace existing code.** You locate the current behavior, its callers, data boundaries, tests, constants, and reusable equivalents.
6. **Resolve order.** You process at most 20 phases with at most 50 tasks and 100 acceptance checks per phase in one execution batch. You continue later batches in plan order until every phase is covered. For a precise task without a plan, you state the bounded implementation sequence in the conversation.
7. **Select checks.** You identify the smallest focused tests for the first change and the required final project gates.
8. **Prepare tracked mode.** When tracked-plan mode applies, you create a dedicated feature branch only when currently on the confirmed default branch, preserve an existing non-default branch, validate the plan status and each documented phase status whether stored in frontmatter or the approved single-file format, and set the plan to `status: in-progress` without a separate status-only commit.
9. **Confirm safety.** You continue only when you can preserve unrelated work, isolate tracked commits from pre-existing changes, and avoid unresolved decisions.

## Stop conditions

- You stop when the plan or task is missing, unreadable, contradictory, or not implementation-ready.
- With neither a readable plan nor a precise inline task, you stop with `plan not found at <path>` and never fabricate one.
- You stop and report an overlap when existing user changes cannot be preserved safely.
- You stop before using a secret, dependency, external service, or destructive operation that is unavailable or unauthorized.
- You stop before tracked-plan mode when the requested status or commit lifecycle cannot be completed without including unrelated changes.

## Test

- The baseline names real target files or explicitly labeled candidate paths.
- Existing user changes are identified and remain untouched.
- The first validation check comes from the repository or the approved plan.
- The selected lifecycle is explicit, and tracked-plan mode starts on a non-default feature branch with the plan marked `in-progress`.
