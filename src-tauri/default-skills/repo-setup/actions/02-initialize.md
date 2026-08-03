# 02 - Initialize

You create local Git metadata on the resolved initial branch without altering project content.

## Input

- Accept a validated non-repository directory and resolved branch name.

## Output

- Return the repository root, unborn `HEAD`, branch, and unchanged project files.

## Process

1. **Recheck boundary.** Confirm that no repository appeared since inspection.
2. **Initialize.** Invoke Git initialization with the resolved branch using separated arguments.
3. **Verify root.** Confirm that the work-tree root equals the requested target and the current branch matches the decision.
4. **Verify content.** Compare project paths and content fingerprints with the pre-initialization inventory. Allow only Git metadata changes.
5. **Continue conditionally.** Continue to `03-bootstrap` only when contribution guidance, a bootstrap commit, or end-to-end publication was explicitly requested.

## Stop conditions

- Never stage, commit, generate documentation, configure identity, or change global Git settings in this action.
- Stop when Git initializes a different root or branch.
- Do not remove partial metadata automatically after a failure. Report the incomplete local state.

## Test

- Confirm that Git reports the requested directory as its root and the resolved branch as current.
- Confirm that no project content was added, edited, deleted, staged, or committed.
