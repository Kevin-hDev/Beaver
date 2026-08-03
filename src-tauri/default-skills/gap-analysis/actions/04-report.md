# 04 - Report

You rank every validated gap and deliver a concise plain or comparative report without changing the source artifact.

## Input

- Accept complete gap batches, optional comparison sets, companion sources, and scope limits.

## Output

- Return ranked gaps, direct questions, coverage, and optional comparison sections.

## Process

1. **Validate consequence.** You remove any item whose absence changes no downstream decision, action, check, or response.
2. **Assign severity.** You use blocker for a hard stop, major for likely rework, and minor for clarity only.
3. **Rank.** You order by downstream phase, severity, affected scope, and dependency on the answer.
4. **Render every batch.** You render at most 30 gaps per batch and continue until every validated gap is reported. You never omit overflow.
5. **Ask directly.** You phrase one minimal answerable question per missing decision and keep its stable key and source anchor visible.
6. **Render comparison.** When comparison sets exist, you render Closed, Still Open, and Newly Introduced in that order. You state whether the previous source was a report or artifact version. You count only Still Open and Newly Introduced as current gaps.
7. **Stamp clean.** You report `status: clean` when no blocker or major remains in Still Open or Newly Introduced, or in the current set during a plain run.
8. **Render intent uncertainty.** You phrase an unconfirmed removal as a missing decision and direct question. You never present it as a deliberate removal or proven regression.
9. **Record coverage.** You list scanned and inapplicable categories, scans completed for each artifact version, processed batch counts, continuation completion, and any missing source that limited confidence.
10. **Deliver safely.** You keep the report in the conversation unless the user requests a file. For a file, you validate the destination, preserve unrelated content, and write only the report.

## Stop conditions

- You never use `blocker` for an item discoverable and resolvable inside the current downstream phase.
- You never hide an unavailable source, incomplete batch, or ambiguous comparison behind a complete verdict.
- You do not rewrite, patch, or silently answer the source artifact.

## Test

- Every reported item changes a downstream decision, action, verification, or interpretation.
- A comparison report contains the three status sections and preserves stable keys.
- A version comparison states that both artifacts were scanned and keeps unconfirmed removal intent uncertain.
- The union of rendered batches contains every validated gap exactly once.
- The report contains no rewritten artifact or unrequested implementation suggestion.
