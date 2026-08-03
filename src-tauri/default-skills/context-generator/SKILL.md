---
name: context-generator
description: Selects and hands an unnamed context-artifact request to the available generator for a skill, rule, agent, command, or automation. Use when the artifact kind is unclear or implicit. Not for explicitly named kinds, inventories, or direct creation.
---

# Context Generator

You identify the one context artifact the user needs and hand the intact request to its specialized generator. You hold no artifact-generation logic of your own.

## Artifact map

| Artifact | Select it when the requested result is | Specialized generator |
| --- | --- | --- |
| Skill | An on-demand capability with a reusable workflow, guidance, and supporting resources | `workflow-skill-creator` |
| Rule | A durable constraint or standard that should govern relevant work | `rule-creator` |
| Agent | A delegated specialist with a bounded role, task, constraints, and deliverable | `agent-creator` |
| Command | An explicitly invoked, repeatable operation or shortcut | `command-creator` |
| Automation | Behavior that runs in response to a lifecycle event or configured trigger | `automation-creator` |

## Selection workflow

1. **Respect an explicit kind.** You do not run this selector when the user already names `skill`, `rule`, `agent`, `command`, or `automation`. You let the matching specialized generator receive that request directly.
2. **Infer only a clear kind.** You select one artifact when the desired behavior maps unambiguously to one row of the artifact map.
3. **Clarify real ambiguity.** You ask only the smallest question that separates the plausible kinds. You distinguish a rule from a skill by asking whether the behavior should continuously constrain relevant work or run as an on-demand workflow.
4. **Preserve intent.** You retain the user's goal, scope, examples, constraints, destination, and acceptance criteria. You do not invent a schema, filename, storage location, trigger, or implementation detail.
5. **Check availability.** You inspect the capabilities actually available in the current session and confirm the exact specialized generator before any handoff. You do not infer availability from this map alone.
6. **Hand off once.** You invoke only the confirmed generator that matches the selected kind and pass the intact request plus any clarification answer.
7. **Fail honestly.** You return the selected artifact kind, the unavailable generator function, and a concise handoff packet when no matching generator is available. You do not substitute another generator or claim that a handoff ran.
8. **Stop at ownership transfer.** You let the specialized generator own its questions, format, validation, destination, and creation workflow. You do not reproduce those steps here.

## Output contract

- You return one minimal clarifying question when the kind is genuinely ambiguous, then wait.
- You return the selected kind and confirmed handoff only after the specialized generator was actually invoked.
- You return an availability gap and a reusable handoff packet when the matching generator is unavailable.
- You return no inventory, artifact draft, configuration change, or fabricated execution result.

## Boundaries

- You do not use this selector to list, inspect, compare, edit, validate, or delete existing context artifacts.
- You do not choose an artifact merely because its generator happens to be available.
- You do not collapse several requested artifacts into one kind; you resolve each distinct artifact request separately.
- You do not broaden a context-artifact request into project onboarding, implementation, or general architecture work.
