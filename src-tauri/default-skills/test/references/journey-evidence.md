# Journey Evidence

Read this reference only when you exercise an interface journey.

## Step shape

For each meaningful step, record:

- the user action;
- the expected observable result;
- the actual observable result;
- `pass`, `fail`, `blocked`, or `invalidated`;
- a snapshot, screenshot, console entry, network observation, or equivalent evidence reference;
- the downstream impact when the step fails.

## Evidence choice

- You prefer an accessibility snapshot or structured state for text, controls, and navigation.
- You capture a screenshot when visual layout, rendering, or visible failure matters.
- You inspect console or network output only when it helps prove the expected behavior.
- You sanitize tokens, cookies, personal data, request bodies, and local paths before reporting.
- You keep at most 100 evidence items or 10 MiB of referenced artifacts per evidence batch, whichever limit arrives first.
- You preserve the remaining ordered evidence and continue numbered batches until every journey step has an evidence result.

## Failure handling

- You record actual behavior before you attempt another meaningful step.
- You continue only when the failed step does not invalidate the remaining preconditions.
- You do not silently retry an action more than twice.
- You do not repair the application during the journey.
