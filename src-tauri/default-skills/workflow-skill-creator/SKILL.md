---
name: workflow-skill-creator
description: Creates or refactors router-based workflow skills with atomic actions, progressive resources, and evaluations. Use for reusable multi-step capabilities. Not for one-shot commands, project rules, CLI/API integrations, or application features.
---

# Workflow Skill Creator

You turn a reusable multi-step procedure into one focused skill bundle, preserve existing work during refactors, and prove the completed bundle against observable cases.

## Workflow

```mermaid
flowchart LR
    Create([Create]) --> Scope["01 - Scope"]
    Modify([Modify]) --> Plan["02 - Plan"]
    Scope --> Plan
    Plan --> Write["03 - Write"]
    Write --> Validate["04 - Validate"]
```

## Actions

You read only the next action file and the references it names before running that step.

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-scope`](actions/01-scope.md) | You are creating a new workflow skill | Confirmed purpose, name, boundary, examples, and destination |
| [`02-plan`](actions/02-plan.md) | You have a confirmed create frame or an existing skill to refactor | Atomic action and resource plan |
| [`03-write`](actions/03-write.md) | The plan and write boundary are confirmed | Complete skill bundle at the confirmed destination |
| [`04-validate`](actions/04-validate.md) | A bundle was written or supplied for validation | Per-file and behavioral validation report |

## Rules

- You require an explicit destination for a new skill and keep an existing skill in place when refactoring it.
- You confirm the name, trigger boundary, exclusions, examples, interaction mode, action plan, and write boundary before changing files.
- You inspect neighboring skills and descriptions for real trigger overlap without treating similar subject matter as automatic duplication.
- You preserve every useful capability in an existing skill unless the user explicitly removes it.
- You create one action per distinct job, give every action an observable test and stop conditions, and keep shared facts in one reference.
- You keep the router concise, load resources progressively, and include only information that can change selection, action, verification, or output.
- You preserve unrelated user content and stop before an unconfirmed overwrite, destination change, semantic reduction, or path escape.
- You report validation as `passed`, `failed`, `blocked`, or `skipped` and never turn missing evidence into success.

## Resources

- Read [destination-resolution.md](references/destination-resolution.md) before confirming or changing a destination.
- Read [naming.md](references/naming.md) before confirming a name or trigger boundary.
- Read [authoring-contract.md](references/authoring-contract.md) before planning or writing files.
- Read [validation-protocol.md](references/validation-protocol.md) before validating the bundle.
- Copy [skill-template.md](assets/skill-template.md) and [action-template.md](assets/action-template.md) only after the plan is confirmed.
- Copy [eval-template.json](assets/eval-template.json) only after concrete trigger and non-trigger cases are confirmed.
