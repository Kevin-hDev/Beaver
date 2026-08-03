# 03 - Review

Review each memory file independently, reconcile findings, and apply only safe evidence-backed corrections.

## Input

- Use the generated or refreshed files, repository evidence, preservation ledger, destination map, and obsolete-file candidates from `02-generate`.

## Output

- Return per-file review findings, applied safe corrections, approval-required findings, duplicate reconciliation, and review coverage.

## Process

1. **Load the protocol.** Read [review-protocol.md](../references/review-protocol.md) and [memory-rules.md](../references/memory-rules.md).
2. **Partition files.** Sort current memory files and review at most 10 files per numbered batch, then continue until every file has one independent result.
3. **Review independently.** Give each file a fresh review pass using only that file, its cited repository evidence, the canonical destination names, and the rules. Do not expose another file's findings before this pass finishes.
4. **Check evidence.** Flag unsupported or stale claims, nonexistent commands or paths, unsupported rationale, missing non-derivable facts, misplaced facts, duplicates, secrets, placeholders, and rule breaches with exact locations.
5. **Reconcile findings.** Merge identical findings after all independent passes. Keep cross-file duplicates assigned to the canonical home from [memory-map.md](../references/memory-map.md).
6. **Apply safe findings.** Apply a correction only when evidence is decisive, the destination is valid, and no pre-existing user text is deleted or rewritten. Apply each file atomically and record the exact finding disposition.
7. **Request approval.** Present any correction that would alter pre-existing user text, move content, remove an obsolete file, or choose between conflicting evidence. Apply only the exact approved findings.
8. **Recheck changed files.** Run a fresh independent pass on every corrected file and confirm duplicate and preservation invariants again.

## Stop conditions

- Stop applying a finding when evidence is ambiguous, user work could be lost, or a destination is outside the bank.
- Stop before deletion unless the user explicitly names the obsolete file to delete.
- Stop with an incomplete review when any selected file cannot receive an independent pass.

## Test

- Confirm that every current memory file has exactly one initial independent review result.
- Confirm that every applied finding cites decisive evidence and preserves unapproved user text byte-for-byte.
- Confirm that every duplicate is removed, moved with approval, or explicitly flagged, never silently ignored.
- Confirm that every changed file passes a fresh review and no file is staged.
