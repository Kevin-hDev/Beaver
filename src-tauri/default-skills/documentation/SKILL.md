---
name: documentation
description: Creates or updates verified READMEs, user/developer guides, API/config references, architecture, migration, operations, and troubleshooting docs. Use for project documentation. Not for specs, ADRs, memory, release notes, or code comments alone.
---

# Documentation

You create accurate project documentation from current behavior, preserve useful existing content, and prove that examples, links, and documented claims remain valid.

## Workflow

```mermaid
flowchart LR
    Request([Documentation request]) --> Scope["01 - Scope"]
    Scope --> Research["02 - Research"]
    Research --> Needed{"Changes needed?"}
    Needed -->|Yes| Write["03 - Write"]
    Needed -->|No| Validate
    Write --> Validate["04 - Validate"]
    Validate -->|Pass| Done([Verified documentation])
    Validate -->|Documentation defect| Write
    Validate -->|Product mismatch| Blocked([Report mismatch])
```

## Actions

Read every action used by the selected path before you run it.

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-scope`](actions/01-scope.md) | You receive a documentation request | A confirmed audience, document type, scope, destination, and success contract |
| [`02-research`](actions/02-research.md) | The documentation contract is clear | An evidence ledger and an update map |
| [`03-write`](actions/03-write.md) | Research proves that documentation changes are required | New or updated project documentation |
| [`04-validate`](actions/04-validate.md) | The documentation draft is complete | Validation evidence and a `complete`, `partial`, or `blocked` result |

## Rules

- You document current verified behavior. You label planned behavior explicitly and never present it as implemented.
- You treat source code, schemas, tests, configuration, generated interfaces, and executable behavior as evidence. You resolve conflicts instead of copying stale prose.
- You use the project's existing documentation structure, terminology, style, and link conventions. You ask when the destination or audience is materially ambiguous.
- You keep all reads and writes inside the canonical project root. You reject traversal, unsafe links, and destinations with unresolved parents.
- You preserve unrelated user content and update the smallest owning document. You do not rewrite a documentation set merely to normalize style.
- You validate commands and examples in a safe fixture, dry run, or documented non-destructive mode when practical. You never execute a destructive setup, migration, deployment, or credential command to prove prose.
- You remove or redact secrets, tokens, personal data, internal stack traces, private endpoints, and machine-specific absolute paths from user-facing documentation.
- You process large documentation sets, source inventories, links, and examples in ordered continuable batches. A batch boundary never becomes a silent total limit.
- You do not implement missing product behavior, change public APIs, redesign architecture, create an ADR, update project memory, prepare release notes, or commit changes unless the user separately requests the appropriate work.
- You return `partial` or `blocked` when required evidence or validation is unavailable. You never convert an unchecked example, unresolved link, or failed documentation build into success.

## Resources

- Read [document-types.md](references/document-types.md) while resolving the document contract.
- Read [evidence-contract.md](references/evidence-contract.md) before researching claims or examples.
- Read [validation-rubric.md](references/validation-rubric.md) before validating the completed documentation.
