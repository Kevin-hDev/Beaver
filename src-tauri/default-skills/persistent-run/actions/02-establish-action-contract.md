# 02 - Establish the action contract

You convert the original request into a precise envelope for autonomous attempts without inventing authority.

## Input

- Accept the initialized tracking record and the user's original request.
- Use the current project state, prerequisite inventory, and known operational costs.

## Output

- Return a recorded action contract with exact scope, allowed effects, exclusions, and bounded totals.
- Return `blocked` when any material contract field cannot be established safely.

## Process

1. You read [the action-contract guide](../references/action-contract.md) and separate explicit authority from technical possibility.
2. You record the files, directories, systems, environments, and data that are in scope. You treat everything else as out of scope.
3. You record permitted operation classes, such as read, edit, create, run local checks, or install a project-local dependency, only when the request reasonably requires each class.
4. You record forbidden or gated effects. You never infer authority for purchases, subscriptions, account creation, credential generation, secret disclosure, destructive actions, external publication, deployment, commits, pushes, tickets, or releases.
5. You include a gated effect only when the original requested workflow explicitly contains that exact effect. You record its target, environment, maximum occurrences, cost ceiling when relevant, rollback or recovery path, and verification before execution.
6. You snapshot relevant uncommitted work and identify files that attempts may touch. You require a non-destructive strategy that preserves unrelated and user-authored changes.
7. You record an overall `max_attempts`, a `batch_size`, a deadline or wall-time budget, bounded command/output limits, and any cost or service quota. You ensure `batch_size <= max_attempts` and a finite remaining total.
8. You record a `no_progress_limit` and measurable progress signals. You use a conservative value when the user did not choose one and expose that value in the contract before work begins.
9. You record the success predicate exactly, including working directory, inputs, expected exit/result, and required evidence. You reject a predicate that can pass for the wrong reason.
10. You ask one focused question only when the user's request does not establish a material scope, effect, boundary, or predicate. You do not ask the user to repeat authority already present in the request.
11. You mark the contract immutable for the current batch. You record later user-approved extensions as amendments; you never silently expand it after a failure.

## Stop conditions

- You stop when the request would require an unrequested external or destructive effect.
- You stop when no finite attempt, time, or resource total can be established.
- You stop when existing work cannot be preserved within the proposed approach.
- You stop when the success predicate is unsafe, non-reproducible, or materially under-specified.

## Test

- You confirm every planned mutation maps to an allowed scope and operation class.
- You confirm the batch and total limits are finite and the no-progress rule is executable.
- You confirm external effects are absent unless they appear exactly in the original requested workflow.
- You confirm the final predicate cannot be replaced by a worker's statement or a proxy check.
