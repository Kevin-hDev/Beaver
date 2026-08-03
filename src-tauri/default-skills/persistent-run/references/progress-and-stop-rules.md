# Progress and stop rules

## Classify evidence

- You classify `step-passed` only when the step's acceptance check passes.
- You classify `progressed` only when a predeclared signal moves measurably toward the predicate without an offsetting regression.
- You classify `no-progress` when the signal is unchanged or the same failure remains.
- You classify `regressed` when a relevant verified condition worsens or a new regression appears.
- You classify `inconclusive` when the check cannot distinguish success from failure.

You count `no-progress`, `regressed`, and `inconclusive` toward the consecutive no-progress threshold. You reset the consecutive count only for verified `progressed` or `step-passed` evidence.

## Change a retry materially

You require at least one evidence-backed change in the suspected cause, diagnostic, target, implementation strategy, tool, or experimental setup. You do not count rewording, a fresh worker, rerunning the same command, or changing unrelated files as a new approach.

## Stop safely

You stop as `blocked` when any of these conditions appears:

- You exhaust the maximum attempts, deadline, wall time, quota, output limit, process limit, or cost limit.
- You reach the no-progress threshold.
- You need user-only information, a secret, unavailable access, or new authority.
- You would need a destructive, financial, credential, account, or external-write effect that is absent from the recorded contract, or the contract's finite occurrence allowance for that effect is exhausted.
- You cannot distinguish attempt changes from user work or preserve that work safely.
- You cannot run a trustworthy verification.

You name the smallest resume condition. You do not promise that the condition will guarantee completion.

You classify the blocker as `input`, `authority`, `access`, `boundary`, `financial`, `destructive`, `external-effect`, `conflict`, or `verification` so a resumed run does not retry the blocked effect blindly.
