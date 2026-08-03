# 01 - Inspect

You establish the exact repository state, workflow choice, and ordered commit concerns without changing the index or working tree.

## Input

- Accept a validated repository root, optional paths, optional message, explicit push choice, and workflow choice `interactive` or `auto`.

## Output

- Return branch, `HEAD`, staged and unstaged groups, relevant untracked paths, ordered concerns, workflow choice, push intent, and required checks.

## Process

1. **Validate repository.** Canonicalize the root, stay inside the allowed workspace, and confirm readable Git metadata and a valid `HEAD` or initial state.
2. **Read instructions.** Load repository rules and commit conventions that govern the requested paths.
3. **Inspect state.** Read status, staged diff, unstaged diff, untracked names, branch, remotes, upstream, and recent message style.
4. **Bound scope.** Stop for a narrower pass when more than 200 changed paths are present. Preserve a continuation list for later passes.
5. **Protect staged work.** Identify pre-existing staged content and whether it belongs to the first requested concern.
6. **Detect unsafe content.** Inspect candidate diffs for credentials, private data, generated artifacts, large binaries, local configuration, and conflicts without reproducing sensitive values.
7. **Group concerns.** Separate changes by user-visible outcome or maintenance responsibility. Preserve an ordered queue when several concerns were requested.
8. **Resolve workflow.** Use the user's explicit choice. Default to `interactive` when a split or message needs approval; use `auto` only when every decision is unambiguous.

## Stop conditions

- Stop outside a repository, on invalid paths, unreadable state, conflicts, suspected secrets, unsupported large files, or unrelated staged content.
- Return `no-change` when nothing within scope can be committed.
- Ask one focused scope question when `auto` would require guessing a concern or hunk split.

## Test

- Confirm that staged, unstaged, and relevant untracked state are accounted for without mutation.
- Confirm that every requested change belongs to one ordered concern or remains explicitly out of scope.
