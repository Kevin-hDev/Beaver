---
name: parallel-work
description: Splits multi-part implementation requests into refined tasks and runs safe, dependency-aware parallel waves, serializing conflicts. Use for explicit fan-out or todo requests. Not for one task or wholly dependent work.
---

# Parallel Work

You turn one multi-part request into refined tasks, execute every ready task with a dedicated executor, and prove both task-level and integrated results.

## Actions

| # | Action | Purpose |
| --- | --- | --- |
| 01 | [Decompose and graph](actions/01-decompose-and-graph.md) | You map the request into complete tasks, dependencies, and exclusive ownership. |
| 02 | [Refine and stage](actions/02-refine-and-stage.md) | You refine every task and build safe parallel waves. |
| 03 | [Execute waves](actions/03-execute-waves.md) | You launch up to six ready executors concurrently and reconcile real results. |
| 04 | [Verify and report](actions/04-verify-and-report.md) | You verify every task, test the integrated state, and return one concise table. |

You read each action file before you perform it. You complete the actions in order and repeat `02` through `04` when verified results reveal new tasks or dependencies.

## Non-negotiable rules

- You preserve every requirement by mapping it to exactly one primary task and any necessary integration check.
- You build a dependency graph before launch. You run only tasks whose predecessors are verified.
- You assign one dedicated executor to each executable task. You make that executor refine its task before implementation.
- You launch at most six executors concurrently in one active session. You keep every remaining task queued under its stable identity and process it in later continuable waves until it reaches a terminal status.
- You parallelize only tasks with exclusive write ownership. You serialize shared-file edits, shared generated artifacts, migrations with ordering constraints, and any other conflicting effects.
- You give every executor the structured contract in [executor-prompt-template.md](assets/executor-prompt-template.md). You never send a vague task summary.
- You inspect the actual files, diffs, commands, tests, and artifacts after every executor returns. You never accept its summary as proof.
- You reconcile the current state before every new wave because completed work may change later assumptions.
- You preserve user-authored and unrelated changes. You never overwrite, reset, or clean them to simplify coordination.
- You perform only effects authorized by the request. You do not create a commit, push, ticket, pull request, release, deployment, account change, or other external effect unless the user explicitly requested that exact effect.
- You continue independent work when one task fails or becomes blocked, then expose the incomplete dependency chain honestly.
- You return exactly one final Markdown table and no surrounding prose.

## Resources

- You read [dependency-and-ownership.md](references/dependency-and-ownership.md) before you create the graph or assign files.
- You read [executor-contract.md](references/executor-contract.md) before you delegate a task.
- You read [integration-verification.md](references/integration-verification.md) before you accept the final state.
- You instantiate [executor-prompt-template.md](assets/executor-prompt-template.md) separately for every launched task.
