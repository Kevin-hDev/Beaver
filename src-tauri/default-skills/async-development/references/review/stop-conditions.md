# Ordered review stop conditions

You evaluate these conditions in order and use the first match.

1. You stop with `blocked-state` when the linked item carries the configured blocked state.
2. You stop with `iteration-limit` when completed correction iterations have reached the configured maximum.
3. You stop with `new-human-feedback` after the first correction iteration when a new human comment is newer than the recorded start of the preceding iteration.
4. You stop with `no-unaddressed-feedback` when no eligible human feedback remains.
5. You continue only when none of the preceding conditions applies.

You treat first-iteration human feedback as the requested correction input, not as an interruption. You classify an author as automated only from verified adapter metadata or a configured bounded allowlist. You never infer automation solely from writing style.

You require a new explicit trigger after `new-human-feedback`, `blocked-state`, or `iteration-limit`. You keep `no-unaddressed-feedback` as a benign, auditable finalization path.

You do not insert an operational failure into this ordered decision list. You preserve a collection, correction, test, audit, reply, resolution, reaction, or branch-drift failure as `critical-failure`, skip further correction, and hand it directly to idempotent finalization for a verified working-to-blocked lock closure. You never let cleanup hide or replace the original failure.
