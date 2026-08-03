---
name: pull-request
description: Opens a user-validated draft or ready pull or merge request for a published branch using project base, label, template, provider, commit, and check rules. Use to create a review request. Not for committing, pushing, merging, or releasing.
---

# Pull Request

You verify the published branch, resolve the correct base from explicit input or project convention, describe the complete committed change, validate the draft with the user, and create one provider-appropriate request.

## Actions

Read only the action required for the current step.

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-inspect`](actions/01-inspect.md) | You receive a pull or merge request request | Provider mechanism, repository, branch, base, publication state, labels, template, and duplicate state |
| [`02-collect`](actions/02-collect.md) | Head and base are unambiguous | Commits, files, behavior, risks, labels, and verified checks since the merge base |
| [`03-draft`](actions/03-draft.md) | The complete change is collected | A user-approved title, body, base, state, and metadata |
| [`04-create`](actions/04-create.md) | Approval and preflight prove creation is safe | Request URL, number, state, head, base, and applied labels |

## Rules

- Never commit, stage, push, create or switch branches, merge, rebase, amend, tag, or modify working files.
- Validate repository root, remote, provider mechanism, current branch, user-supplied base, project branch convention, and requested metadata.
- Resolve base in this order: valid explicit base, documented branch-prefix mapping, confirmed repository default branch. Never assume `main` or `master`.
- Require a non-default head whose remote counterpart contains local `HEAD`.
- Search for an existing open request with the same repository, head, and base before creating another.
- Describe the full committed difference from the merge base and exclude unrelated working-tree changes.
- Use repository contribution rules and request template before the bundled fallback.
- Create a draft by default. Create a ready request only when the user explicitly requests ready or non-draft.
- Derive a triage label from a documented branch-prefix mapping and apply it only when that label exists. Add other metadata only when explicitly requested.
- Show title, body, head, base, state, mapped label, and explicit metadata. Wait for user approval before external creation.
- Use any available authenticated provider connector, CLI, MCP capability, or API. Never ask for or expose credentials.
- Stop on ambiguous base, unpublished head, unavailable provider, duplicate request, failed mandatory gate, conflicting state, or rejected draft.
- Report only remote state confirmed by the provider and stop after request creation.

## Resources

- Read [branch-conventions.md](references/branch-conventions.md) when the project defines no complete base or label mapping.
- Read [request-content.md](references/request-content.md) when the repository provides no usable request template.
- Copy [pull-request-template.md](assets/pull-request-template.md) only as the fallback body structure.
