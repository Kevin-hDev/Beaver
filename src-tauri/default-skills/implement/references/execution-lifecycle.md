# Execution Lifecycle

Select one lifecycle before editing and keep it for the complete requested scope.

## Workspace mode

- Use this mode when the request asks for implementation without plan-status or Git-history outcomes.
- Preserve all pre-existing changes and leave implementation edits uncommitted.
- Report the final diff, checks, and repository state.

## Tracked-plan mode

Use this mode only when an approved plan, project rule, or explicit user request requires the complete lifecycle.

1. Resolve the default branch and current branch without guessing.
2. Create a dedicated feature branch only when the current branch is the default branch. Keep an existing non-default branch.
3. Move the plan from `pending` to `in-progress` before the first phase. Let this marker ride in the first phase commit.
4. Move each phase from `pending` to `in-progress` while working, then to `done` only after its acceptance checks pass.
5. Commit each phase exactly once with its code, tests, documentation, and `done` status. Exclude unrelated changes.
6. On a human-only blocker, set the plan to `blocked`, commit the bounded blocker state, and stop.
7. After every phase is `done` and the final clean sweep passes, set the plan to `implemented` and create one final lifecycle commit.

Never create a separate `in-progress` commit. Never push, publish, or open a pull request unless the request explicitly extends the endpoint to that external state.

## Atomicity checks

- Inspect staged paths before every commit.
- Confirm every staged path belongs to the current phase or required lifecycle marker.
- Confirm no phase-owned edit remains after its commit.
- Keep user-owned and unrelated changes unstaged and unchanged.
