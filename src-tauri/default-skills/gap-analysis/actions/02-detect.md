# 02 - Detect

You identify consequential missing decisions, behavior, and evidence without inventing their answers.

## Input

- Accept the framed artifact and relevant authoritative companion sources.

## Output

- Return ordered gap batches with stable identity, evidence, consequence, question, and severity.

## Process

1. **Scan actors.** You check roles, affected users, ownership, access, and responsibility relevant to the artifact.
2. **Scan states.** You check initial, loading, empty, success, partial, failure, recovery, disabled, and terminal states that matter.
3. **Scan failures.** You check invalid input, unavailable dependency, timeout, conflict, retry, rollback, and error communication where relevant.
4. **Scan boundaries.** You check scope edges, limits, permissions, platforms, localization, accessibility, privacy, and compatibility when applicable.
5. **Scan data.** You check source, validation, lifecycle, ownership, consistency, migration, retention, and deletion when the artifact depends on data.
6. **Scan dependencies.** You check external decisions, systems, teams, ordering, assumptions, and operational preconditions.
7. **Scan verification.** You check observable completion, acceptance, evidence, measurement, and failure verdicts.
8. **Scan assumptions.** You check conditions treated as agreed or obvious without authoritative support.
9. **Scan ambiguities.** You check terms with multiple reasonable interpretations that change downstream behavior.
10. **Assign identity.** You read [gap-identity.md](../references/gap-identity.md) and assign each gap a stable key from its category, source anchor, and normalized missing decision. You never use severity or question wording in the key.
11. **Deduplicate.** You merge symptoms with the same stable key and discard preferences or already resolved information.
12. **Batch.** You order gaps by downstream consequence, process at most 30 per batch, carry a continuation ledger, and continue until every consequential gap in scope is covered.
13. **Keep version scans independent.** When this action scans an earlier version for comparison, you use the shared frame and rubric but only that version's evidence. You never let the current version silently fill an earlier omission or let earlier prose fill a current omission.

## Stop conditions

- You do not report a category that cannot change downstream behavior or understanding.
- You do not infer a missing requirement merely because the artifact uses a different structure or wording.

## Test

- Every gap names a stable key, missing information, downstream consequence, direct question, and source anchor or absent section.
- The set contains no duplicate, solution proposal, or already answered question.
