---
name: commit
description: Creates one or more atomic Git commits with project conventions, interactive or automatic workflow choices, and optional explicit push. Use to commit, split concerns, or push. Not for amending, rebasing, branching, review requests, or releases.
---

# Commit

You preserve unrelated work, divide the requested change into atomic concerns, and repeat `inspect → collect → message → record` once per concern. You select either the `interactive` or `auto` workflow.

## Workflow choices

- **Interactive:** Show each proposed concern and message. Wait for approval before staging a split and before recording its commit.
- **Auto:** Continue without prompts only when scope, split, message, and push intent are already unambiguous. Stop and ask when judgment would change included content or history.

## Actions

Read only the action required for the current step.

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-inspect`](actions/01-inspect.md) | You receive a commit request | Repository state, workflow choice, concerns, scope, and checks |
| [`02-collect`](actions/02-collect.md) | One concern is selected | One atomic staged set and remaining concern queue |
| [`03-write-message`](actions/03-write-message.md) | The staged diff is final | A project-compliant message approved when interactive |
| [`04-record`](actions/04-record.md) | The staged set and message pass preflight | A commit SHA, optional explicit push, and continuation state |

## Rules

- Treat all existing changes and staged content as user-owned. Never discard, reset, restore, clean, stash, or rewrite them.
- Validate the repository root and every requested path, reject traversal, and inspect at most 200 changed paths per pass.
- Inspect status, staged diff, unstaged diff, relevant untracked paths, branch, remotes, and recent message style before staging.
- Stage only explicit paths, explicit hunks, already staged content, or one clearly coherent requested concern.
- Preserve existing staged files. Stop instead of unstaging them when they conflict with the selected concern.
- Split multiple requested concerns into multiple commits and repeat the workflow. Never bundle them for convenience.
- Follow the project's commit convention and use the bundled reference only when none exists.
- Never bypass hooks, sign with unavailable credentials, amend, rebase, create a branch, or open a review request.
- Never use `--force`. Use `--force-with-lease` only when the same request explicitly requires a non-fast-forward push and the verified lease makes the update safe.
- Push only when the original request explicitly includes it. Use a normal push whenever it can succeed.
- Never expose or include secret values, private data, generated noise, or local credentials.
- Report only checks, commit state, remaining concerns, and remote state actually observed.
- Stop after the requested commits and optional push. Do not continue into review or release work.

## Resources

- Read [commit-message.md](references/commit-message.md) only when the project defines no consistent convention.
