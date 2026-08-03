# 04 - Approve

You obtain explicit user approval for the exact learning packet and delivery effects.

## Input

- Use the complete scored recommendations, reconciliation evidence, destination conventions, and capability states from `03-assess`.
- Accept the user's approvals, edits, destination changes, or rejections.

## Output

- Return complete approved learning packets with exact destination, reconciliation action, affected files or handoff, synchronization plan, and approval evidence.
- Return covered, rejected, and deferred candidates in a no-write ledger.

## Process

1. **Show the proposal.** You present the lesson, evidence, score, reconciliation, destination, exact intended effect, supersession links, review step, and post-write synchronization when applicable.
2. **Wait for approval.** You ask the user to approve, edit, redirect, or reject each proposed packet. You treat silence, prior source approval, and general encouragement as no approval.
3. **Reassess changes.** You return to `03-assess` when an edit or destination change affects score, reconciliation, convention, supersession, or delivery capability.
4. **Build packets.** After explicit approval, you copy [learning-packet.md](../assets/learning-packet.md), remove its guidance, and fill every field from approved evidence.
5. **Close no-ops.** You keep `covered`, rejected, and deferred candidates out of delivery. When the user requests a change to covered content, you return it to `03-assess` and classify the supported change as `updates` before seeking fresh approval.
6. **Approve prerequisites separately.** When a memory destination requires a missing bank, you ask separately whether to hand a complete setup request to the available project-memory capability. You record that approval as a prerequisite handoff, not as approval or delivery of the lesson.

## Stop conditions

- You stop before packet creation when approval is ambiguous, partial, or attached to a different destination.
- You stop when the exact affected files, handoff generator, supersession links, or synchronization targets remain unknown.
- You never transform approval for one packet into approval for another.

## Test

- Every deliverable packet contains explicit approval, source evidence, score, destination, reconciliation, scope, intended effect, and synchronization state.
- No `covered`, rejected, deferred, or unapproved candidate enters delivery; any requested change to covered content is reassessed as `updates` first.
- Any material change to an approved packet receives fresh approval.
