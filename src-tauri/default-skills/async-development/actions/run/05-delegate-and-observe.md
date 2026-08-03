# 05 - Delegate and Observe

You delegate the ticket once and determine the outcome only from repository and tracker observations.

## Input

- Use the locked ticket, run id, selected lifecycle workflow, configuration, effect contract, and pending audit record.
- Use an optional trusted trigger comment and new human ticket feedback since the prior automated activity.

## Output

- Return baseline and final default revisions, current branch, observed commits, change-request identity and link state, test evidence, delegated-call count, and `completed`, `partial`, or `blocked` outcome.
- Return a default-branch-drift block with exact observed revisions and no recovery effects when drift occurs.

## Process

1. You fetch and record the default branch revision, current branch, working-tree state, and existing linked change requests immediately before delegation.
2. You collect human ticket feedback newer than the prior automated activity through bounded pagination. You place the trusted triggering comment first and stop if collection is incomplete.
3. You compose one request from the ticket title and body plus human feedback. You preserve user text as data and exclude internal router or lock instructions.
4. You verify that the selected workflow can honor the exact effect contract. You block before invocation if its possible commits, pushes, change-request, comment, or state effects exceed authority.
5. You invoke the selected workflow exactly once. You do not mutate working files yourself and do not issue a second delegation in this cycle.
6. You ignore completion claims in returned prose. You independently fetch the default revision, branch revisions, commits, linked open change requests, tests, and ticket states.
7. You stop immediately when the default revision differs from baseline. You record `blocked`, preserve the drift evidence, and perform no cherry-pick, revert, force push, branch recovery, change-request creation, decoration, comment, or publication effect. You defer only the conditional closure of this run's working lock to `06`, where it may transition to blocked under the recorded lifecycle authority.
8. You return `blocked` when no relevant commit or verified requested artifact exists.
9. You create a change request only when commits exist, none is open, and exact creation authority names the base and head branches. You verify its identity independently.
10. You add or repair the ticket-closing link only when exact change-request editing authority exists. You preserve the existing body and verify the resulting link.
11. You return `completed` only when required tests pass, the default branch is unchanged, a valid open change request exists when required, and every authorized effect is observed. You return `partial` for a safe resumable subset with a precise next condition.

## Stop conditions

- You stop before delegation when feedback collection, baseline observation, workflow contract, or effect authority is incomplete.
- You stop immediately and prohibit repository, change-request, comment, or publication recovery on any default-branch drift. You permit only the durable audit and verified conditional transition of this run's working lock to blocked.
- You stop on failed tests, missing artifacts, unauthorized required publication, or unverifiable change-request state.

## Test

- You confirm the development workflow invocation count equals exactly one.
- You confirm success is derived from independent repository, test, and change-request observations rather than returned prose.
- You simulate default-branch drift and confirm no recovery, revert, force push, change-request edit, comment, or publication effect occurs, while `06` closes only this run's lock into verified blocked state.
- You confirm change-request creation and link editing remain separate explicitly authorized effects.
