# 01 - Inspect

You establish whether one new pull or merge request can be created without modifying Git history.

## Input

- Accept a validated repository root and optional base, title, body, draft state, reviewers, labels, milestone, assignees, project, or provider preference.

## Output

- Return provider mechanism, repository identity, head, base and reason, publication state, project templates, mapped label, explicit metadata, and duplicate state.

## Process

1. **Validate repository.** Canonicalize the root, stay inside the workspace, and confirm readable Git metadata.
2. **Read rules.** Load contribution instructions, request templates, VCS configuration, branch conventions, and label mappings that govern the repository.
3. **Identify provider.** Use the project-declared provider when present; otherwise infer the host and repository from a validated remote. Select an available authenticated connector, CLI, MCP capability, or API.
4. **Resolve head.** Reject detached `HEAD` and record local and remote head SHAs.
5. **Resolve base.** Use a valid supplied base, else a documented branch-prefix mapping, else the provider-confirmed default branch. Surface the selected base and evidence.
6. **Check head role.** Reject the resolved base as head unless the provider explicitly supports and the user requests a same-branch comparison.
7. **Resolve label.** Map the head prefix to one project triage label when documented. Verify that the label exists; otherwise skip it without error.
8. **Verify publication.** Confirm that remote head exists and contains local `HEAD`.
9. **Find duplicates.** Search open requests for the same repository, head, and base.
10. **Inspect local state.** Report uncommitted changes as excluded without staging or changing them.

## Stop conditions

- Stop on invalid remote, unavailable provider mechanism, detached `HEAD`, missing or ambiguous base, unpublished local head, or conflicting repository identity.
- Return the existing URL instead of creating a duplicate.
- Never configure a remote, upstream, credential, or provider account.

## Test

- Confirm that one provider, repository, head, and base are supported by observed project or provider evidence.
- Confirm that remote head contains local `HEAD` and no matching request exists.
