---
name: playbook
description: Manages reusable project playbooks by listing, researching, creating, updating, and applying them with verified sources and tracked execution. Use for repeatable procedures. Not for one-off implementation, general docs, or unapproved changes.
---

# Playbook

You maintain reusable, evidence-backed project procedures and apply them only through an explicitly confirmed, observable checklist.

## Actions

Read each selected action before you run it.

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-list`](actions/01-list.md) | You need to discover available project playbooks and packaged examples | A stable numbered inventory with active and shadowed entries |
| [`02-upsert`](actions/02-upsert.md) | You need to create or substantially update a project playbook | A contract-compliant playbook based on verified research |
| [`03-research`](actions/03-research.md) | You need current alternatives, missing coverage, or surprising improvements | A sourced, read-only research report and recommendation |
| [`04-apply`](actions/04-apply.md) | You need to execute an existing playbook against the current project | A confirmed execution ledger, verification evidence, and remaining work |

Run `list` first when no playbook is named. Run `research` before creating or substantially updating a playbook. Keep every action independently usable.

## Rules

- You resolve project playbook locations from the user's explicit destination or one unambiguous existing project convention. You ask before inventing a directory.
- You treat [packaged examples](assets/examples/) as read-only. You write only to an explicitly resolved project location.
- You keep project reads and writes inside the canonical project root. You reject traversal, unresolved parents, symlink escapes, and personal or global destinations that were not explicitly selected.
- You resolve playbooks by the latest numbered list, exact slug, exact title, then topic match. You show candidates instead of guessing an ambiguous match.
- You preserve useful user-authored content, research before material changes, detect overlap, and never silently overwrite a project playbook.
- You verify current claims against primary or official sources. You use community evidence only as adoption or operational signal, not as proof that a tool or behavior exists.
- You make research read-only. You do not write a playbook until the user requests `upsert` or explicitly accepts a handoff to it.
- You analyse an application before changing state, show the exact effects, and obtain explicit confirmation. You do not execute human-only, unsupported, destructive, credential, account, or out-of-scope steps.
- You track every selected application step as `pending`, `in-progress`, `verified`, `blocked`, `skipped`, or `human`. You continue in ordered batches until every selected step reaches a terminal status.
- You verify observable outcomes and return `partial` or `blocked` with a resumable checkpoint when evidence, access, or a required human action is missing. You never convert an unchecked result into success.

## Resources

- Read [locations.md](references/locations.md) for discovery, resolution, precedence, and write ownership.
- Read [playbook-contract.md](references/playbook-contract.md) before creating or updating a playbook.
- Read [research-method.md](references/research-method.md) before researching a topic or material update.
- Use [playbook-template.md](assets/playbook-template.md) as the output scaffold.
- Use [research-goal-checklist.md](assets/research-goal-checklist.md) and [research-completion-checklist.md](assets/research-completion-checklist.md) to gate research.
