# 03 - Compare

You compare current gaps with a previous gap report or with gaps derived from a previous artifact version.

## Input

- Accept the complete current gap batches from `02-detect`.
- Accept an optional previous gap report or previous artifact version as text or from a validated path.
- Accept the shared artifact frame when the previous source is an artifact version.

## Output

- Return complete Closed, Still Open, and Newly Introduced sets with stable keys, source type, and explicit intent uncertainty.

## Process

1. **Validate the previous source.** Reject traversal, unreadable content, an unauthorized path, empty content, or a source that cannot be parsed safely. Read long inputs in bounded chunks until complete.
2. **Classify the source.** Distinguish a structured previous gap report from an earlier artifact version. Never parse ordinary requirement prose as if it already contained gap keys.
3. **Scan an earlier artifact.** When the source is an artifact version, apply `01-frame` with the current purpose, consumer, boundary, and categories. Run `02-detect` completely against the earlier version using the same rubric and identity rules. Preserve earlier evidence separately from current evidence.
4. **Handle the first run.** When no previous source exists, classify every current gap as Newly Introduced and leave Closed and Still Open empty.
5. **Recover prior identities.** For a previous report, parse every stable key and preserve category, anchor, missing decision, question, severity, and evidence. For an earlier artifact, use the stable keys produced by its scan.
6. **Normalize semantic anchors.** Read [gap-identity.md](../references/gap-identity.md). Match the same capability, decision, or policy across renamed sections, changed actors, and reworded sentences when the underlying missing decision remains the same.
7. **Match identities.** Match current and previous gaps by stable key. Ignore question rewording and severity changes. Never match by list position or exact sentence alone.
8. **Classify resolved gaps.** Place a previous-only key in Closed only when the current artifact or an applicable authority explicitly resolves the missing decision or confirms that its capability is deliberately out of scope.
9. **Classify shared and new gaps.** Place shared keys in Still Open and current-only keys in Newly Introduced. Keep previous wording for Closed and current wording for Still Open and Newly Introduced.
10. **Preserve removal uncertainty.** When an earlier explicit requirement, actor, field, or behavior disappears or is replaced, do not call it intentionally removed, resolved, or regressed without evidence. Create or preserve a current ambiguity gap for removal intent, cite both versions, and ask whether the change is deliberate. Remove that uncertainty only when the user or an authoritative source confirms intent.
11. **Handle legacy reports.** When a previous report lacks stable keys, fall back to category plus normalized semantic anchor and missing decision, or category plus quoted evidence when nothing stronger exists. Mark ambiguous matches unresolved instead of guessing.
12. **Batch completely.** Compare at most 30 identities per batch, keep a continuation ledger, and continue until both sets and every removed explicit requirement are exhausted.

## Stop conditions

- Stop without comparison when the previous source is malformed, the artifact purposes are incompatible, or identity cannot be recovered safely.
- Do not classify an ambiguous legacy match as Closed or Newly Introduced merely because wording changed.
- Do not infer intent from omission, role replacement, weaker wording, or document silence.
- Never modify either artifact or the previous report.

## Test

- Confirm that an earlier artifact version receives the same complete scan as the current version.
- Confirm that a reworded question with the same stable key remains Still Open.
- Confirm that a previous-only gap becomes Closed only with current resolution or confirmed scope removal.
- Confirm that an unconfirmed removed requirement remains an explicit uncertainty with both source anchors.
- Confirm that the three sets cover every unique previous and current key exactly once.
