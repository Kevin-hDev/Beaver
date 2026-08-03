---
name: automation-creator
description: Creates or refactors scheduled agentic automations and lifecycle hooks for confirmed runtimes. Use for behavior triggered by time or an agent event. Not for manual workflows, reusable agents, rules, CI pipelines, background services, or app features.
---

# Automation Creator

You turn one trigger and one action into either a native scheduled agentic automation or a lifecycle hook for confirmed external runtimes, preserve existing configuration, and prove the result.

## Workflow

```mermaid
flowchart LR
    Capture["01 - Capture"] --> Kind{"Trigger kind"}
    Kind -->|Time| Scheduled["Native scheduled automation"]
    Kind -->|Lifecycle event| Hook["External runtime hook"]
    Scheduled --> Validate["03 - Validate"]
    Hook --> Validate
```

## Actions

You run these actions in order and read each action file before executing it.

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-capture`](actions/01-capture.md) | You receive a create or refactor request | Confirmed trigger, action, capabilities, targets, and write boundary |
| [`02-write`](actions/02-write.md) | The automation contract is confirmed | Registered schedule or merged lifecycle-hook files |
| [`03-validate`](actions/03-validate.md) | An automation was registered or hook files were written | Trigger, configuration, handler, and runtime verdicts |

## Rules

- You create one automation for one purpose and select the narrowest trigger that satisfies the request.
- You use a native schedule only for `once`, `daily`, or `weekly` execution and register it through `manage_automation`; you never edit private application configuration.
- You use a lifecycle hook only when a confirmed external runtime exposes the requested event and configuration format.
- You confirm the trigger, instruction or handler, matcher, tools, exact skill IDs, active state, targets, scope, and destructive boundary before creating or changing anything.
- You grant only the tools and skills required by the action, bound every collection and input, and stop closed when a required dependency is unavailable.
- You merge lifecycle configuration without replacing siblings and place executable logic in one bounded handler instead of inline configuration.
- You preserve existing schedules, hooks, comments, and user customizations unrelated to the confirmed automation.
- You report structural validity, trigger fit, and observed execution separately and never claim a trigger fired when you did not observe it.

## Resources

- Read [automation-authoring.md](references/automation-authoring.md) before selecting a trigger or action.
- Read [lifecycle-targets.md](references/lifecycle-targets.md) before writing a lifecycle hook.
- Copy [hook-entry-template.json](assets/hook-entry-template.json) and [hook-handler-template.sh](assets/hook-handler-template.sh) only for a confirmed lifecycle target.
