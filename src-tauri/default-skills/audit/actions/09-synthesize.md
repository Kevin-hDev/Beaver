# 09 - Synthesize

You deduplicate the selected pillar results and produce stable pillar reports plus one ranked merged report with honest coverage.

## Input

- Use findings, unverified candidates, executed checks, skipped checks, and inspected scope from every selected pillar.

## Output

- Return stable finding identities, per-pillar results, a verdict, continuable merged batches of at most 50 ranked findings, top actions, positive verified controls, and coverage limits.

## Process

1. **Validate findings.** You remove preferences, unsupported claims, duplicates, and results outside the selected scope.
2. **Merge causes.** You combine findings with the same cause and preserve all affected locations without exceeding 10 locations per row.
3. **Assign identities.** You derive `AUD-<pillar>-<digest>` from the pillar slug plus the normalized primary relative path, stable symbol or manifest key, and violated control or root cause. You use the first 12 lowercase hexadecimal characters of SHA-256 over those UTF-8 fields separated by a null byte. You preserve the same identity across every batch and report and reject identity collisions.
4. **Rate consistently.** You apply the shared severity and effort rubric across pillars.
5. **Rank impact.** You order confirmed findings by severity, reachability, user impact, confidence, then finding identity.
6. **Separate uncertainty.** You keep probable or unverified candidates outside the confirmed findings table.
7. **Record coverage.** You list every selected pillar as scanned, partially scanned, or skipped with the actual checks and reasons.
8. **Finish all batches.** You complete every numbered pillar batch before synthesis. You emit at most 50 confirmed findings in one merged report batch, preserve the full order and remaining count, and continue numbered batches until every confirmed finding appears exactly once.
9. **Return report.** You return the merged report in the conversation by default.
10. **Write requested artifacts.** When the user requests report artifacts, you use [audit-pillar-template.md](../assets/audit-pillar-template.md) for `code-quality.md`, `architecture.md`, `security.md`, `dependencies.md`, `performance.md`, `tests.md`, or `ui.md` as selected, and [audit-report-template.md](../assets/audit-report-template.md) for `report.md`. You render every file, verify complete identity coverage and no placeholders, stage each file beside its destination, and replace each file atomically. You fail closed before replacement if any file is invalid and never overwrite unrelated files.

## Stop conditions

- You never mark the project healthy when a selected pillar was skipped without making the limitation visible.
- You never include more than 50 merged findings in one batch or hide overflow; you continue later batches until no finding remains.
- You never write a partial artifact set while a pillar batch remains or when identity coverage differs between pillar and merged reports.
- You never imply that a suggested fix was implemented or tested.

## Test

- Every confirmed finding has one unique stable identity, severity, pillar, evidence, impact, recommendation, and effort.
- Every requested pillar report contains exactly its pillar identities, and `report.md` contains every confirmed identity exactly once across its numbered batches.
- Every selected pillar appears in coverage, and no audited project file changed except explicitly requested report artifacts.
