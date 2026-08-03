# 05 - Deliver

You independently review and apply each approved packet through its permitted destination path.

## Input

- Use complete approved learning packets, current destination files, resolved conventions, and confirmed generator availability.

## Output

- Return each packet with destination, `created`, `updated`, `covered`, `handed-off`, `unavailable`, or `failed` action, exact file or generator result, and independent review verdict.

## Process

1. **Revalidate current state.** You re-read every affected file and recheck generator availability before preparing an effect. You stop when the approved reconciliation no longer matches current content.
2. **Prepare the smallest change.** You preserve unrelated content and render only the approved learning into the narrowest resolved destination.
3. **Handle a missing memory bank.** You pass an explicitly approved bank-setup request only to a project-memory capability confirmed available. When it succeeds, you return to `03-assess` to resolve the new convention and seek fresh lesson approval; you never claim that the setup handoff delivered the lesson. When the capability is unavailable or fails, you return the complete setup request as `unavailable` or `failed`.
4. **Prepare memory delivery.** You target an existing project-local memory entry, or a new entry only when an existing memory-bank convention explicitly defines its location and format. You never create the bank itself.
5. **Prepare ADR delivery.** You use the project's existing ADR template or [adr-template.md](../assets/adr-template.md). For `supersedes`, you render the new ADR's `Supersedes` link and the older ADR's `Superseded by` link together.
6. **Prepare rule or skill handoff.** You pass the complete approved packet only to a specialized generator confirmed available for that destination. When none is available, you return the complete packet with action `unavailable` and do not write a rule or skill directly.
7. **Review independently.** You read [independent-review.md](../references/independent-review.md) and perform a fresh evidence-to-packet-to-output review without relying on the earlier recommendation. You return `pass`, `changes-required`, or `blocked`.
8. **Resolve review findings.** You correct only mechanical rendering defects that leave the approved packet unchanged and rerun review. You return to `04-approve` before any semantic, destination, or supersession change.
9. **Apply a direct write.** After a `pass` verdict, you validate canonical destinations, render all affected memory or ADR files to sibling temporary files, verify current originals and all links, then replace each destination atomically. You write only after approval and stage nothing.
10. **Apply a handoff.** After a `pass` verdict, you invoke the confirmed specialized generator with the approved packet, capture its returned result, and never claim success when it fails or cannot verify its output.
11. **Report delivery.** You account for every approved packet and name changed files, handoff result, no-op, failure, review verdict, and whether synchronization remains required.

## Stop conditions

- You stop before any effect when a file changed since assessment, a path escapes the project, a link is invalid, review does not pass, or the generator is unavailable or fails.
- You do not write personal or global memory, create a missing memory bank, write rules or skills directly, or stage files.
- You do not replace a superseded ADR without the bidirectional links prepared and validated together.

## Test

- Every direct write has explicit approval, a `pass` review, a resolved project convention, and an atomic validated replacement.
- Every ADR supersession links old to new and new to old.
- Every rule or skill result names the confirmed generator, or returns the complete approved packet as `unavailable` without a false application claim.
- Every missing-bank setup either returns to assessment after a verified project-memory handoff or reports `unavailable` or `failed` without claiming lesson delivery.
- Every approved packet appears exactly once in the delivery summary.
