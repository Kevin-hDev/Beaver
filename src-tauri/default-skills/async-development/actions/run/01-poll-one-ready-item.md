# 01 - Poll One Ready Item

You select at most one eligible ticket for this cycle and preserve a cursor for the remaining queue.

## Input

- Use the validated configuration and tracker adapter.
- Accept an optional normalized trigger event and exact ticket hint.

## Output

- Return zero or one selected ticket with stable identity, title, body, URL, states, revision, and trigger context.
- Return the deterministic ordering rule, pages inspected, continuation cursor, and excluded-item reasons.

## Process

1. You validate the configuration digest, adapter identity, repository target, trigger type, and any ticket hint.
2. You fetch only the hinted ticket when a trusted event supplies one. You otherwise query open tickets in the configured ready state.
3. You paginate with the configured page size and maximum pages. You stop with `incomplete` and a cursor rather than claiming the queue is exhausted when the bound is reached.
4. You exclude items already in working or blocked state and items with an open linked change request.
5. You order eligible items by the configured deterministic priority and stable identity tie-breaker.
6. You select the first eligible item only. You retain a continuation cursor or remaining-work indicator for a later invocation.
7. You perform no state, comment, branch, or file mutation in this action.

## Stop conditions

- You stop when the adapter response is malformed, unbounded, or cannot prove item identity and revision.
- You stop with `incomplete` when the collection bound is reached before a deterministic selection can be justified.
- You stop the cycle cleanly when no eligible item exists.

## Test

- You confirm a hinted event fetches only its exact ticket.
- You confirm a queue query returns at most one selected item and a resumable continuation state.
- You confirm working, blocked, and already-linked items are excluded.
- You confirm this action performs no external mutation.
