# 03 - Acquire Lock

You acquire one atomic lifecycle lock so concurrent triggers cannot process the same ticket.

## Input

- Use the ready ticket with its observed revision, validated configuration, tracker adapter, and effect contract.
- Use the adapter's documented conditional mutation or compare-and-set primitive.

## Output

- Return the ticket identity, lock status, run id, idempotency key, previous revision, new revision, and verification evidence.
- Return `contended` without mutation when another run owns the lock.

## Process

1. You require exact authority to change lifecycle state on the selected ticket and to write the pending audit at the configured project path. You stop before locking when either authority is absent.
2. You generate a bounded unpredictable run id and stable idempotency key without using ticket text as executable input.
3. You re-read the ticket revision and states immediately before locking.
4. You return `contended` when the working state is present, the ready state is absent, the ticket revision changed incompatibly, or a lock owner already exists.
5. You require a documented conditional update that atomically adds working state, removes ready and review states when present, and binds the update to the observed revision or absent lock.
6. You refuse to emulate atomic locking with independent sequential state edits.
7. You read the ticket again and verify the working state, removed trigger states, new revision, and run ownership marker.
8. You write the lock observation into a new durable pending audit record atomically before delegation.

## Stop conditions

- You stop when the adapter lacks an atomic conditional primitive.
- You stop the cycle cleanly on contention without retrying in the same invocation.
- You stop and mark the run blocked when lock mutation or independent verification fails.

## Test

- You confirm two concurrent attempts cannot both return `lock_acquired` for one ticket revision.
- You confirm a contended attempt leaves the ticket unchanged.
- You confirm the acquired lock contains the run ownership marker and excludes trigger states.
- You confirm the pending audit record exists before any development delegation.
