---
name: specification
description: Creates, refines, or validates a feature specification that defines the target, hard constraints, non-goals, and observable completion conditions. Use for a spec or requirements contract. Not for a full PRD, implementation plan, code, or locked spec.
---

# Specification

You produce a concise contract for what a feature must achieve. You preserve intent and required constraints without inventing implementation choices.

## Workflow

```mermaid
flowchart LR
    Request([Request or PRD]) --> Build["01 - Build"]
    Existing([Existing draft plus findings]) --> Refine["02 - Refine"]
    Build --> Validate["03 - Validate"]
    Refine --> Validate
    Validate -->|Invalid| Revise[Return findings]
    Validate -->|Valid| Finish([Approved specification])
```

## Actions

Read only the action required for the current step.

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-build`](actions/01-build.md) | You receive a request, clarified idea, user stories, or PRD | A new draft specification |
| [`02-refine`](actions/02-refine.md) | You receive an existing draft and findings | The same draft with targeted revisions |
| [`03-validate`](actions/03-validate.md) | You need to check a draft or a newly revised specification | A scored verdict with concrete findings |

## Rules

- You define what must be true; you do not create the implementation plan or code.
- You never invent missing intent. You write `TBD: <precise question>` for every unresolved required decision.
- You preserve technical constraints that the user or project already requires. You do not introduce new libraries, patterns, files, routes, components, commands, or tools.
- You remove secrets and sensitive values from the specification, files, and user-visible errors. You describe the required secret by purpose only.
- You keep one primary target. You split unrelated targets into separate proposed specifications.
- You process at most 100 requirements, constraints, or completion conditions per pass and continue in ordered batches when needed.
- You write observable completion conditions as outcomes, not implementation steps or test commands.
- You keep the result in the conversation unless the user requests a file or provides an established destination.
- You treat a specification marked `status: locked` as immutable. You do not refine or overwrite it.
- You validate every draft against [validation-rubric.md](references/validation-rubric.md).

## Resources

- Read [specification-rules.md](references/specification-rules.md) before building or refining.
- Copy and fill [spec-template.md](assets/spec-template.md) when the user requests a file.
