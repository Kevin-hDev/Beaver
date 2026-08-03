---
name: persistent-run
description: Pursues a measurable success condition through bounded attempts and resumable tracking. Use when the user asks to keep trying until an observable predicate passes. Not for one-shot work, subjective goals, monitoring, or unlimited autonomy.
---

# Persistent Run

You pursue a measurable outcome through bounded attempts, independent verification, and a durable project-local record. You never translate persistence into unlimited authority or an unsupported claim of success.

## Actions

| # | Action | Purpose |
| --- | --- | --- |
| 01 | [Initialize or resume](actions/01-initialize-or-resume.md) | You validate the goal, load or create tracking, and map the journey. |
| 02 | [Establish the action contract](actions/02-establish-action-contract.md) | You turn the request into exact scope, effects, and attempt boundaries. |
| 03 | [Run an attempt](actions/03-run-attempt.md) | You execute one hypothesis-driven attempt and gather direct evidence. |
| 04 | [Evaluate and finish](actions/04-evaluate-and-finish.md) | You verify progress, continue within limits, complete, or stop safely. |

You read an action file before you perform that action. You start with `01`; you then apply `02` before any mutation and repeat `03` and `04` until a terminal state is justified.

## State contract

You keep one tracking file at a user-selected location, an established project task-log location, or the announced fallback `.persistent-runs/<task-slug>.md`. You use [the tracking template](assets/tracking-template.md) and these states:

- You use `pending` before the first attempt.
- You use `in-progress` while an attempt may continue within the recorded contract.
- You use `blocked` when completion requires human input, new authority, unavailable access, or a larger recorded boundary.
- You use `completed` only when the recorded success predicate has just passed and `completion: verified` cites direct evidence.

You treat the append-only attempt log as the history of record. You preserve earlier entries, user-authored project changes, and any tracking-file content outside marked generated sections.

## Non-negotiable rules

- You require an observable command or deterministic predicate, a total attempt limit, a deadline or time budget, bounded resource use, and a no-progress threshold before you mutate anything.
- You infer authority only from the user's actual request. You treat silence as no authority for external effects, account changes, secrets, destructive work, purchases, commits, pushes, tickets, releases, or deployments.
- You never repeat the same failed hypothesis. You record what materially changed before every retry.
- You verify each attempt independently from its executor's claim. You rerun the final predicate yourself.
- You stop at the first contract boundary, no-progress threshold, hard blocker, unsafe conflict, or exhausted total. You do not convert a bounded batch into an unlimited total.
- You resume only after you reconcile the tracking record with current files and preserve changes made since the last attempt.

## Resources

- You read [the action-contract guide](references/action-contract.md) when you establish or extend authority.
- You read [the progress and stop rules](references/progress-and-stop-rules.md) before you retry a failure.
- You use [the attempt log format](references/attempt-log-format.md) for every attempt.
- You read [the verification guide](references/verification.md) before you claim completion.
- You give a delegated executor [the bounded worker prompt](assets/attempt-worker-prompt.md) when isolated workers are available; otherwise you apply the same contract directly.
