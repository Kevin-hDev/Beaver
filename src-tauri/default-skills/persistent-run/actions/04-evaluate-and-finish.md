# 04 - Evaluate and finish

You classify the evidence, detect stagnation, prove success, or stop with a resumable blocked state.

## Input

- Accept the latest attempt entry, tracking record, success predicate, and current project state.
- Use the progress, verification, no-progress, and boundary rules recorded in the contract.

## Output

- Return `completed`, `in-progress`, or `blocked` with direct evidence and the exact next condition.
- Return a verified completion record or a resumable blocker report without false success.

## Process

1. You read [the progress and stop rules](../references/progress-and-stop-rules.md) and classify the attempt as `step-passed`, `progressed`, `no-progress`, `regressed`, or `inconclusive`.
2. You check a step only when its own acceptance criterion passes independently. You leave it unchecked for progress, inference, or partial output.
3. You update the progress fingerprint and consecutive no-progress count. You count regressions and inconclusive results as no progress unless the contract defines a stricter rule.
4. You stop the current batch when it reaches its batch size. You continue with another batch only while the recorded total, deadline, resources, and no-progress threshold still permit it and the log justifies a changed hypothesis.
5. You set `blocked` when the no-progress threshold or any total boundary is reached, or when completion needs human input, new authority, access, funds, a destructive choice, or an unrequested external effect. You record a blocker kind, what was tried, what evidence blocks progress, and the smallest condition that would permit resume.
6. You return to the next attempt when a distinct hypothesis remains inside every boundary. You never reset counters or erase failed attempts to manufacture room.
7. You run the exact success predicate after all planned steps pass or whenever evidence indicates the objective may already be satisfied. You run it from its recorded directory with its recorded inputs.
8. You read [the verification guide](../references/verification.md), inspect the predicate for false positives, and retain bounded direct output, result codes, timestamps, and relevant state identity.
9. You set `status: completed` and `completion: verified` only after the predicate has just passed under the current state. You record the completion evidence and stop immediately.
10. You treat a failed final predicate as new root-cause evidence, not as completion or a reason to rerun a checked step unchanged. You append an unchecked diagnostic or remediation step with a falsifiable hypothesis and direct acceptance check, then continue only when that step fits the existing contract and every remaining boundary.
11. You set `blocked` when a failed final predicate yields no distinct in-contract hypothesis or when adding the required step would exceed authority, time, attempt, resource, or no-progress limits. You record the predicate failure and the smallest resume condition.
12. You leave `completion: unverified` for every other outcome. You never use phrases such as guaranteed, fixed, or done when the predicate failed, was skipped, or could not be reproduced.

## Stop conditions

- You stop successfully only after the exact predicate passes and current evidence is recorded.
- You stop as `blocked` at a no-progress threshold, exhausted boundary, authority gap, human-only input, unsafe conflict, or inconclusive final verification.
- You stop without retrying an effect that could charge money, destroy data, publish externally, or overwrite user work.
- You stop when continuation would require changing the success predicate to make existing work appear successful.

## Test

- You confirm every checked step has direct acceptance evidence.
- You confirm the final predicate was rerun after the last relevant change and cannot pass for an unrelated reason.
- You confirm a failed final predicate creates a new unchecked evidence-driven step or a precise blocked record.
- You confirm `completed` always pairs with `completion: verified` and a reproducible evidence record.
- You confirm every blocked record names a smallest resume condition and preserves remaining history.
