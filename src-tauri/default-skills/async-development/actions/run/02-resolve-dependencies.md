# 02 - Resolve Dependencies

You resolve every declared dependency source in order before implementation begins.

## Input

- Use the selected ticket, validated configuration, tracker adapter, and effect contract.
- Use native dependency relations, configured textual conventions, and lifecycle states.

## Output

- Return `ready` or `blocked`, every open blocker identity, the first blocking source, and verification evidence.
- Return any authorized blocked-state or explanatory-comment effect with independent verification.

## Process

1. You query documented native dependency relations first and paginate within configured bounds.
2. You parse the ticket body for configured dependency declarations only. You validate every referenced identifier before fetching it.
3. You inspect the configured blocked state as the final fallback.
4. You stop at the first dependency source that proves an open blocker, but you collect every blocker from that source within the bound.
5. You return `incomplete` instead of `ready` when any dependency page, identifier, or state cannot be verified.
6. You apply the configured blocked state only when the exact tracker mutation is authorized. You verify the final state through an independent read.
7. You post one bounded blocker comment only when that exact ticket comment is authorized. You use a stable idempotency marker and verify the posted comment.
8. You fail closed on an authorized state or comment failure. You record known completed effects for safe resume.

## Stop conditions

- You stop the cycle when any verified blocker remains open.
- You stop when dependency collection is incomplete or a referenced item cannot be resolved safely.
- You stop on the first failed authorized state or comment effect.

## Test

- You confirm an open native dependency blocks before textual fallback is considered.
- You confirm an open configured textual dependency blocks and closing it permits readiness on a later cycle.
- You confirm an incomplete page cannot produce `ready`.
- You confirm denied comment authority produces no comment while the blocked decision remains accurate.
