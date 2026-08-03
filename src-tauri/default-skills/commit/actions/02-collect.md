# 02 - Collect

You stage exactly one selected concern while preserving every unrelated change and queued concern.

## Input

- Accept the inspected repository state, workflow choice, selected concern, and remaining concern queue.

## Output

- Return one atomic staged set, its concern, and the unchanged remaining queue.

## Process

1. **Preserve the index.** Keep existing staged content unchanged when it belongs to the selected concern.
2. **Propose the split.** In `interactive`, show the concern, paths or hunks, exclusions, and reason. Wait for approval before staging. In `auto`, proceed only when the split is unambiguous.
3. **Select narrowly.** Stage explicit validated paths or isolated hunks. Never use a repository-wide staging shortcut that can include unreviewed content.
4. **Handle untracked files.** Add an untracked file only when it is in scope, reviewed, non-sensitive, and not generated noise.
5. **Verify the complete staged diff.** Read file statistics, renames, modes, binary entries, and every relevant hunk.
6. **Check atomicity.** Confirm that every staged change supports the same concern.
7. **Check exclusions.** Confirm that unrelated work and later concerns remain unstaged and untouched.

## Stop conditions

- Stop when concerns cannot be separated safely or hunk staging would mix unrelated lines.
- Stop when staging would overwrite user-owned index state.
- Stop when the staged diff is empty, conflicted, or suspected to contain a secret.
- Never unstage, restore, discard, stash, or edit a working-tree file in this action.

## Test

- Confirm that the staged diff contains one reviewed concern.
- Confirm that every unrelated and queued change remains unchanged.
