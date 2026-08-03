# 10 - Record Repository Changes

You commit and push only generated paths whose exact repository effects the user authorized.

## Input

- Use the confirmed configuration, effect contract, generated-path inventory, and current repository state.
- Accept separate explicit authority for committing and for pushing a named branch.

## Output

- Return the reviewed paths, commit status and revision, branch, push status, and independent remote verification.
- Return resumable manual steps when either effect is denied.

## Process

1. You rebuild the generated-path list from verified action outputs. You reject paths outside the repository and exclude unrelated or pre-existing user changes.
2. You inspect the working tree and show a bounded status and diff for only those paths.
3. You stop if the default branch revision differs from the detection baseline and the requested operation could overwrite or conceal concurrent work.
4. You require explicit commit authority before staging. You stage only the reviewed path list through separate validated arguments.
5. You verify the staged diff matches the reviewed generated content and contains no credential value.
6. You create one project-conformant commit only when authorized and capture its revision.
7. You require separate push authority for the exact remote and branch. You never interpret commit authority as push authority.
8. You push without force and independently verify the remote branch contains the captured revision.
9. You return exact safe manual steps when an effect is denied. You do not execute a combined shell string.

## Stop conditions

- You stop when unrelated paths are staged, generated files changed after review, or sensitive data appears.
- You stop before commit or push when its separate authority is denied or ambiguous.
- You stop when the remote branch moved unexpectedly or push verification fails.

## Test

- You confirm a denied commit creates no staged change or commit.
- You confirm an authorized commit includes only the reviewed generated paths.
- You confirm a denied push leaves the remote unchanged even after an authorized commit.
- You confirm an authorized push is verified from the remote revision and never uses force.
