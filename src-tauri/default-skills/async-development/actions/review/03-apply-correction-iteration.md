# 03 - Apply Correction Iteration

You delegate one correction round, verify the result, and respond to each addressed feedback item through exact authorized effects.

## Input

- Use the complete feedback collection, exact change request, discovered lifecycle workflow, configuration, effect contract, review lock, and iteration history.
- Use the project's verified test commands and current change-request branch.

## Output

- Return the iteration number, single delegation count, baseline and final revisions, commit identity, test result, addressed feedback identifiers, reply and resolution observations, and next-loop state.

## Process

1. You resolve the change-request head and base branches and record their remote revisions before mutation. You preserve unrelated working-tree changes and stop on an unsafe checkout.
2. You discover one complete development workflow through the same capability contract used by the run sub-flow.
3. You compose one correction request containing every eligible non-automated comment, stable id, source, path, line, and diff context. You require all listed feedback and prohibit unrelated scope expansion.
4. You ensure the workflow can honor exact commit, push, comment, and change-request boundaries. You invoke it exactly once for this iteration.
5. You independently fetch base and head revisions after delegation. You stop immediately with no recovery effects when the default or base branch changed unexpectedly.
6. You run the verified project test suite with bounded output and time. You never mark feedback addressed when required tests fail.
7. You do not perform an inner second delegation. You append the bounded failure evidence so a later authorized iteration can use a changed correction hypothesis while preserving the total iteration limit.
8. You require exact commit and push authority when the workflow did not already produce those required effects. You verify the new head commit from the remote change request.
9. You post one idempotent reply for each successfully addressed feedback item only when exact comment authority exists. You use a threaded reply for inline review feedback and the documented source surface for other feedback. You verify every reply.
10. You resolve each eligible review thread only when exact resolution authority exists and the verified fix addresses it. You verify `resolved` independently and fail closed on any error.
11. You update the trigger reaction only when separately authorized. You verify the terminal reaction and never suppress an error.
12. You append the iteration start, feedback ids, observation evidence, commit, tests, replies, resolutions, and failure state atomically to the durable audit record.
13. You convert every terminal correction failure after lock acquisition into a `critical-failure` finalization input for `04`. You perform no further correction, reply, resolution, reaction, or publication effect, but you do not abandon the run-owned working lock without the finalizer's verified blocked transition.

## Stop conditions

- You stop before delegation when branch state, feedback, workflow, or effect boundaries are incomplete.
- You stop immediately on base-branch drift and perform no recovery, reply, resolution, reaction, comment, or publication effect. You permit only the durable failure audit and `04`'s verified conditional transition of this run's working lock to blocked.
- You stop on failed tests, missing remote commit, failed reply, failed resolution, failed reaction, or audit-write failure.

## Test

- You confirm the lifecycle workflow invocation count equals one for the iteration.
- You confirm failed tests leave feedback unaddressed and preserve evidence for a later bounded iteration.
- You confirm every reported reply and thread resolution is independently observable.
- You simulate base-branch drift and confirm no recovery, reply, resolution, reaction, comment, or publication effect occurs beyond the verified working-to-blocked lock closure.
- You confirm every terminal correction failure reaches `04` with the original error intact and cannot leave a silently owned working lock.
