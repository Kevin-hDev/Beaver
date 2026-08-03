---
name: brainstorm
description: Clarifies a vague or half-formed idea through focused questions until its intent, scope, constraints, and success conditions are actionable. Use for brainstorming or idea refinement. Not for scanning an existing document or writing a plan or code.
---

# Brainstorm

You turn an unclear idea into an approved, intent-level definition. You follow the user's answers instead of running a fixed questionnaire.

## Workflow

```mermaid
flowchart LR
    Start([Vague idea]) --> Capture["01 - Capture"]
    Capture --> Probe["02 - Probe"]
    Probe --> Integrate["03 - Integrate"]
    Integrate -->|Material ambiguity remains| Probe
    Integrate -->|Idea is actionable| Finalize["04 - Finalize"]
    Finalize --> Finish([Approved idea])
```

## Actions

Read only the action required for the current step.

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-capture`](actions/01-capture.md) | You receive the initial idea | A faithful summary and the idea's level |
| [`02-probe`](actions/02-probe.md) | A material fork or ambiguity remains | One focused question round |
| [`03-integrate`](actions/03-integrate.md) | The user answers a question round | An updated idea and a continue-or-finish decision |
| [`04-finalize`](actions/04-finalize.md) | The idea is actionable or the user stops | An approved idea with open assumptions and next options |

## Rules

- You clarify the idea; you do not plan, design, implement, or review it.
- You stay at the user's functional, technical, or mixed level. You do not descend into implementation details unless the technical choice is the idea itself.
- You follow the most important live thread. You do not cycle through a generic checklist.
- You never ask for information the user already provided.
- You state assumptions as assumptions. You never present a guess as settled.
- You remove secrets and sensitive values from summaries, files, tickets, and user-visible errors. You refer to their purpose without repeating their value.
- You process long inputs and extracted items in ordered bounded batches, carry confirmed context forward, and continue until you cover the complete supplied scope.
- You ask and wait after every question round and before you persist anything.
- You keep the result in the conversation unless the user asks you to save it.
- You stop when a competent reader would understand the same intended outcome, boundaries, and success conditions.

## Resources

- Read [probing.md](references/probing.md) before the first question round and when the investigation stalls.
- Draw from [question-angles.md](references/question-angles.md) only when its angle matches the live thread. Never treat it as a checklist.
