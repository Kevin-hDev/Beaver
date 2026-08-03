---
name: rule-creator
description: Creates or refactors focused, enforceable project coding rules from a stated topic or repository evidence, then writes and validates every confirmed local target. Use for rules, conventions, or standards. Not for skills, code, or temporary notes.
---

# Rule Creator

You turn a coding convention into one canonical rule, render it only to confirmed project-local destinations, and prove every intended target remains consistent.

## Workflow

```mermaid
flowchart LR
    Request([Rule request]) --> Capture["01 - Capture"]
    Capture --> Confirm{"Topic, scope, and destinations confirmed?"}
    Confirm -->|No| Capture
    Confirm -->|Yes| Write["02 - Write"]
    Write --> Validate["03 - Validate"]
    Validate --> Result{"Every intended target passes?"}
    Result -->|Yes| Done([Complete])
    Result -->|No| Blocked([Partial or blocked])
```

## Actions

You read each action file before running that step and complete its tests before continuing.

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-capture`](actions/01-capture.md) | You receive a topic or a request to discover rule candidates | Confirmed canonical rule plan |
| [`02-write`](actions/02-write.md) | You have a confirmed topic, scope, and destination set | Prepared and safely written target files |
| [`03-validate`](actions/03-validate.md) | You have intended target files or a rule to check | Per-target and cross-target verdicts |

## Rules

- You resolve existing project-local rule, instruction, naming, scope, and mirroring conventions before proposing a destination.
- You ask the user to choose the destination, format, and mirroring behavior when project evidence is absent, conflicting, or ambiguous.
- You never invent a destination, impose a directory taxonomy, or select one active instruction surface silently.
- You require written confirmation of the topic, category or local grouping, slug or local filename, file scope, canonical wording, and every intended target before writing.
- You preserve unrelated user content and obtain explicit scope before replacing or substantially restructuring an existing rule.
- You write one focused topic per rule and split crowded requests only after confirming the split.
- You express enforceable behavior as concise second-person imperative statements and add small good or bad examples only when they remove ambiguity.
- You maintain one canonical meaning across every target that the project convention or user requires, while adapting only destination-specific metadata and syntax.
- You validate paths, content, scope metadata, conflicts, examples, and cross-target equivalence before reporting success.
- You report each intended target as `passed`, `failed`, `blocked`, or `skipped`, and never turn missing evidence into a successful verdict.

## Resources

- Read [destination-resolution.md](references/destination-resolution.md) before selecting or confirming targets.
- Read [rule-authoring.md](references/rule-authoring.md) before drafting canonical content.
- Read [consistency-validation.md](references/consistency-validation.md) before writing or validating multiple targets.
- Copy [rule-template.md](assets/rule-template.md) only after the rule plan is confirmed.
