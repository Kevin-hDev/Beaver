# 02 - Plan

You turn the confirmed frame or requested refactor into atomic actions and progressively loaded resources.

## Input

- Use the confirmed create frame or the current existing skill, requested changes, and preservation boundary.

## Output

- Return a confirmed plan listing every action, its input, output, dependencies, stop conditions, test, required references, assets, metadata, and evaluations.

## Process

1. **Read the contract.** You read [authoring-contract.md](../references/authoring-contract.md) and inspect every existing file affected by a refactor.
2. **List distinct jobs.** You identify the separate jobs required to produce the observable result. For a refactor, you start from existing behavior and mark only the jobs the user authorized to change.
3. **Atomize actions.** You create one action per distinct job, merge actions that share the same input, output, and validation, and keep a single inline workflow when no separation is useful.
4. **Place shared knowledge.** You assign each reusable fact to one reference, each copied output scaffold to one asset, and each deterministic repeated operation to one script only when it earns the extra file.
5. **Design evaluations.** You convert the confirmed cases into positive, negative, invalid-input, tool-failure, no-change, and observable-result checks. You add a semantic judge whenever keywords cannot prove success.
6. **Check dependency order.** You ensure no action consumes output from a later action and mark every decision or loop that belongs in the router.
7. **Confirm the plan.** You present the plan and wait for confirmation before writing new files or changing existing ones.

## Stop conditions

- You stop when an action has no observable output, test, or stop condition.
- You stop when the plan weakens existing behavior, duplicates one fact across files, or adds a resource with no concrete consumer.
- You return to scoping when the name, destination, trigger boundary, or overwrite scope must change.

## Test

- You verify that every action owns one distinct job and has a complete input-to-output contract.
- You verify that every resource has at least one named consumer and every confirmed case maps to an evaluation.
- You verify that a refactor preserves all behavior outside the confirmed change boundary.
