---
name: software-workflow
description: Orchestrates a software change through specification, mandatory planning, delegated implementation, independent review, iteration, and requested delivery. Use for end-to-end development. Not for one isolated stage or unrequested publication.
---

# Software Workflow

You orchestrate one software change from its available sources to independently reviewed work. You invoke specialized capabilities for their complete workflows instead of reproducing their internal logic.

## Workflow

```mermaid
flowchart LR
    Request([Request and sources]) --> Spec["01 - Spec"]
    Spec --> Plan["02 - Plan"]
    Plan --> Implement["03 - Implement"]
    Implement --> Review["04 - Review"]
    Review -->|iterate| Implement
    Review -->|ship| Delivery{"Delivery requested?"}
    Delivery -->|No| Handoff([Verified handoff])
    Delivery -->|Yes| Ship["05 - Ship"]
    Ship --> Done([Delivered change])
```

## Actions

You read the selected action before you run it. You run `01 → 02 → 03 → 04`, loop `04 = iterate → 03 → 04`, and run `05` only when the original request explicitly includes delivery. You may invoke any action directly when its required upstream artifacts and evidence are available.

| Action | Use it when | Output |
| --- | --- | --- |
| [01 - Spec](actions/01-spec.md) | You must consolidate the request and its sources | A normalized contract or an exact justified skip |
| [02 - Plan](actions/02-plan.md) | You have an objective and acceptance criteria | A mandatory validated implementation plan |
| [03 - Implement](actions/03-implement.md) | You have a validated plan | Delegated work with observed validation evidence |
| [04 - Review](actions/04-review.md) | The planned work is implemented | An independent `ship` or `iterate` verdict |
| [05 - Ship](actions/05-ship.md) | The current change passed review and delivery was requested | A verified commit and change request, as authorized |

## Orchestration rules

- You resolve specialized capabilities at runtime from their descriptions and current availability. You select capabilities for specification, planning, implementation, review, commit, branch publication, and change-request creation only when their described contract matches the current step.
- You let each selected capability execute and own its complete workflow, artifact, and validation. You never replace a specialized action with your summary of what that action would have done.
- You write the plan in the orchestration context through the planning capability. You never let an implementation worker invent, rewrite, or approve its governing plan.
- You delegate implementation to an executor. You delegate review to a different, independent checker that did not implement the reviewed change.
- You inspect artifacts, command exit codes, repository state, diffs, validation output, and remote state yourself. You never accept a worker's completion statement as proof.
- You maintain the statuses `pending → in-progress → implemented → reviewed`, or `blocked`, in one workflow ledger. You keep the ledger in the conversation unless the user selects a project-local destination or the project already establishes one.
- You preserve unrelated user changes and validate every path, source, command argument, branch, remote, and external identifier before use.
- You never create or switch branches implicitly. You never commit, publish, or open a change request unless the original request explicitly includes that delivery.
- You run implementation and review iterations in continuable bounded waves. You impose no arbitrary total-wave ceiling, but you stop on missing authority, a human-only decision, unsafe state, or demonstrated absence of progress.
- You stop after a fresh successful review with a verified handoff when delivery is not requested.

## Resources

- You read [orchestration evidence](references/orchestration-evidence.md) before you delegate work, assess a return, or decide freshness.
- You copy and fill [the workflow ledger template](assets/workflow-ledger-template.md) only when a project-local ledger is explicitly selected or established.
