# 04 - Record

You create one commit, perform only the explicitly requested push, and continue with the next queued concern when requested.

## Input

- Accept the verified staged set, final approved message, required checks, workflow choice, explicit push choice, expected upstream SHA, and remaining concern queue.

## Output

- Return the new commit SHA, subject, branch, remaining repository state, hook result, push result, and next queued concern.

## Process

1. **Run required checks.** Execute only repository-mandated checks for this commit and stop on failure.
2. **Recheck the index.** Confirm that staged diff and `HEAD` have not changed since message approval.
3. **Create the commit.** Pass the message without a shell-constructed command and allow configured hooks to run.
4. **Handle hook changes.** Inspect hook-created modifications. Retry at most once only when they affect the same staged scope and remain safe.
5. **Verify the commit.** Read SHA, subject, parent, changed paths, and remaining status.
6. **Push conditionally.** Push only when the original request includes it. Use a normal push first when valid.
7. **Lease conditionally.** Use `--force-with-lease=<remote-ref>:<observed-sha>` only when the user explicitly requested a non-fast-forward update, the remote ref and expected SHA were freshly verified, no protected-branch or project rule forbids it, and the update affects only the intended branch. Never use bare `--force-with-lease` or `--force`.
8. **Verify remote state.** Confirm that the remote branch has the reported SHA. Leave a successful local commit intact when push fails.
9. **Continue the queue.** When more requested concerns remain, return to `02-collect`. In `interactive`, wait for approval of the next split. In `auto`, continue only while it remains unambiguous.

## Stop conditions

- Never use `--no-verify`, amend, rebase, bare force, destructive recovery, implicit upstream configuration, or implicit credentials.
- Stop when a hook changes unrelated files, rejects the commit, or fails after one safe retry.
- Stop before a lease push when the expected remote SHA changed or safety cannot be proven.

## Test

- Confirm that new `HEAD` matches the reported SHA and contains one concern.
- Confirm remote tracking state for every reported push.
- Confirm that remaining concerns are preserved and the workflow can continue without bundling them.
