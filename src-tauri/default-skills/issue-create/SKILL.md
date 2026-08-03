---
name: issue-create
description: Creates one actionable, non-duplicate tracker issue from verified evidence with available provider tooling, metadata, and user approval. Use to file a bug, feature, task, or documentation issue. Not for reading, editing, or bulk creation.
---

# Issue Create

You turn one concrete request into one non-duplicate issue. You preserve project templates, source fields, explicit metadata, and user control over the final external mutation.

## Actions

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-resolve`](actions/01-resolve.md) | You receive an issue creation request | Tracker mechanism, project, type, template, fields, and duplicates |
| [`02-gather`](actions/02-gather.md) | Required context needs evidence | Reproduction or use case, outcome, impact, solution status, constraints, references, and validation |
| [`03-draft`](actions/03-draft.md) | Enough evidence exists | A complete user-approved title, body, and metadata |
| [`04-create`](actions/04-create.md) | The exact draft is approved | Created issue URL, identifier, and verified metadata |

## Rules

- Resolve any available authenticated tracker connector, CLI, MCP capability, or API from project configuration or validated repository metadata. Never ask for or expose credentials.
- Validate tracker project, issue type, title, URLs, paths, references, attachments, and external content.
- Search open and recently closed issues for duplicates before creation.
- Create one issue per run and split unrelated problems before drafting.
- Ask only for missing information that changes reproduction, expected behavior, scope, destination, or required fields.
- Use project templates and contribution rules before the bundled fallback.
- Preserve source fields for objective or problem, proposed solution when supported, technical constraints, references or attachments, and QA or validation.
- Describe observed facts separately from reports and hypotheses. Never invent reproduction, environment, impact, solution, or validation.
- Add labels, type, assignee, milestone, project, priority, or attachments only when explicitly requested or mandatory and valid.
- Never attach secrets, personal data, raw logs, local-only paths, or unsafe files.
- Show the exact title, body, labels, type, project, milestone, assignee, priority, attachments, and destination. Wait for explicit user approval before creation.
- Report only the issue and metadata confirmed by the tracker. Stop after creating it.

## Issue quality contract

- For a bug, include actual behavior, expected behavior, minimal reproduction, impact, known frequency, and safe evidence.
- For a feature, include affected user, current limitation, desired outcome, boundaries, and observable completion criteria.
- For a task, include a concrete deliverable, dependencies, exclusions, and verification.
- For a documentation issue, include audience, misleading or missing content, source of truth, and expected correction.
- Treat an issue as an exact duplicate only when behavior or outcome, affected scope, and usable current status match.
- Treat an issue as related when it shares a component or cause but differs in outcome, scope, environment, or required work.
- Keep a proposed solution optional unless the project already selected it.

## Resources

- Copy [issue-template.md](assets/issue-template.md) only when the project provides no applicable template.
