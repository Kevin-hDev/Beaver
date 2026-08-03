# 01 - Collect Feedback

You acquire the review-cycle state and collect every new discussion item within explicit pagination bounds.

## Input

- Use the exact change-request identity, validated configuration, effect contract, adapter records, and optional trigger comment identity.
- Use the last review iteration audit timestamp or accept an explicit validated `since` timestamp.

## Output

- Return the linked ticket, iteration number, review-cycle lock revision, source-tagged comments, pages and cursors inspected, addressed identifiers excluded, collection completeness, and any early lock-release observation.
- Return each authorized reaction or lifecycle transition with independent verification.

## Process

1. You resolve the change request and its linked ticket through adapter relations. You require exact authority for the review lock and configured project audit path, and you stop when authority or either identity is ambiguous.
2. You locate the durable audit record by exact change-request identity and read its complete bounded iteration history. You start at iteration one when no prior record exists.
3. You acquire a conditional review-cycle lock that removes ready-for-review and awaiting-review states and adds working state only when exact lifecycle authority exists. You verify lock ownership and revision atomically.
4. You add a start reaction to the exact trigger comment only when that reaction is separately authorized. You verify it and fail closed on error.
5. You fetch inline review comments, top-level change-request comments, and linked-ticket comments from their documented adapter endpoints.
6. You paginate every stream with configured page size and maximum pages. You preserve source, stable id, author metadata, body, timestamp, path, line, diff context, and thread identity when available.
7. You return `incomplete` with per-stream cursors when any bound is reached before exhaustion. You do not continue to correction with a partial collection.
8. You classify automated authors from verified metadata and the configured bounded allowlist. You do not classify from content style alone.
9. You filter comments newer than `since` and exclude identifiers already recorded as addressed in a verified prior iteration.
10. You place the trusted triggering comment first when present and return the complete ordered collection.
11. You conditionally restore the exact pre-cycle review state and remove working state when any collection, pagination, audit, or authorized reaction failure stops the action after lock acquisition. You release only when the lock revision and owner still match this run, verify the resulting revision and states independently, and record the failure and release observation atomically. You return `blocked` with a precise manual resume condition when lock release cannot be verified.

## Stop conditions

- You stop when the change request, linked ticket, audit record, or review lock cannot be resolved safely.
- You stop on an incomplete discussion stream, preserve its continuation cursor, and close this run's review lock before returning.
- You fail closed when an authorized lock or reaction cannot be verified. After lock acquisition, you perform only the verified conditional lock release and audit update before returning.

## Test

- You confirm all three configured discussion streams paginate to exhaustion or return `incomplete` explicitly.
- You confirm previously addressed identifiers are excluded without dropping newer replies.
- You confirm automated-author classification uses metadata or the configured allowlist.
- You confirm the review-cycle lock is atomic, belongs to this run, and is independently verified.
- You confirm every early stop after lock acquisition either restores the verified pre-cycle review state or reports an unverified release as a blocking manual condition.
