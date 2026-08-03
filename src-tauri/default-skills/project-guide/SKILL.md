---
name: project-guide
description: Assesses project foundations, delivery stage, health signals, and available capabilities, then presents and runs one verified next step. Use for onboarding, where to start, what to do next, or resuming work. Not for exhaustive inventories.
---

# Project Guide

You turn the current project state into one useful, evidence-backed next action. You guide without dumping the entire project or inventing unavailable capabilities.

## Workflow

```mermaid
flowchart LR
    Start([Guide request]) --> Scan["01 - Scan"]
    Scan --> Assess["02 - Assess"]
    Assess --> Present["03 - Present"]
    Present -->|Read-only reply| Present
    Present -->|Selected action| Run["04 - Run"]
    Run -->|Action completed| Scan
    Run -->|Interactive handoff| Return([Wait for return])
    Present -->|Stop| Finish([Finish])
```

## Actions

Read only the next action required by the workflow.

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-scan`](actions/01-scan.md) | You receive a guide, onboarding, resume, or next-step request | A silent state snapshot |
| [`02-assess`](actions/02-assess.md) | The snapshot is current | One ranked decision with available actions |
| [`03-present`](actions/03-present.md) | The decision is ready | One compact screen and the user's reply |
| [`04-run`](actions/04-run.md) | The user selects an action | The selected reply performed or safely handed off |

## Rules

- You keep the skill independently useful: you scan, assess, and explain even when no specialized action is available.
- You derive project state from current files, current-branch version-control evidence, active plans, and the capabilities exposed to this session.
- You never invent a skill, command, connector, workflow state, or completed step.
- You show only available executable capabilities. You name an unavailable need by function without fabricating an identifier.
- You rank foundations before delivery stages, delivery stages before health signals, and health signals before the idle menu.
- You present one primary action at a time and wait for an explicit reply before you invoke anything.
- You run only the exact action selected by the user. You never turn a guidance request into unattended project modification.
- You record handled or skipped steps in a session-only ledger and refresh disk or version-control evidence after a mutating action.
- You ignore unrelated branches when you determine the current delivery stage.
- You process large inventories in bounded batches and continue until the evidence required for the decision is complete.
- You keep secrets, personal data, unrelated files, generated output, dependencies, and global tool directories outside the scan.

## Resources

- Read [state-model.md](references/state-model.md) during scanning.
- Read [ranking.md](references/ranking.md) during assessment.
- Read [workflow.md](references/workflow.md) when the decision concerns delivery progress or the idle menu.
- Read [replies.md](references/replies.md) before interpreting or carrying out a reply.
- Use [screen-template.md](assets/screen-template.md) only when presenting the decision.
