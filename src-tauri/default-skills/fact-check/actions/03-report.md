# 03 - Report

You rewrite the checked text for the reader while keeping corrections, conflicts, uncertainty, and citations visible.

## Input

- Accept the original text, complete verification ledger, and skip ledger.

## Output

- Return complete corrected prose, nearby citations, unresolved claims, observable skipped-verification reasons, sources, and an optional memory suggestion.

## Process

1. **Confirm coverage.** You reconcile selected claims, verification results, and skipped facts. You stop before reporting if any selected claim lacks a result, any skipped fact lacks a reason, or a batch remains unfinished.
2. **Preserve structure.** You keep the original intent and organization unless a false claim requires a local correction.
3. **Cite supported claims.** You place a descriptive link or project evidence next to the claim it supports.
4. **Correct contradictions.** You replace a disproved statement with the supported fact and cite it without repeating misinformation as true.
5. **Surface conflicts.** You state each supported side and explain the unresolved applicability difference without inventing consensus.
6. **Mark uncertainty.** You label unresolved claims clearly and state what source or scope is missing.
7. **Show skips.** When factual statements were skipped, you add a concise `Skipped verification` section containing each statement and its concrete reason. You omit the section when nothing was skipped.
8. **Audit citations.** You remove sources that are indirect, inaccessible, mismatched, duplicated without value, or unsupported.
9. **Respect source limits.** You paraphrase and quote only the minimum necessary within applicable limits.
10. **Suggest memory optionally.** When one or more verified facts are stable, reusable, and supported by durable project or primary evidence, you may append one concise optional memory suggestion naming the fact and source. You ask before any later persistence and perform no write or external action in this skill.

## Stop conditions

- You never present a source list detached from the claims it supports.
- You do not expose internal verification traces, raw search output, secret values, or private data.
- You do not deliver a complete report while any continuation batch remains unfinished.
- You do not hide why a factual statement was skipped or use a skip label for an unresolved selected claim.
- You do not persist, cache, or hand off a suggested fact automatically.

## Test

- Every corrected or supported consequential claim has nearby direct evidence.
- Conflicted and unresolved claims remain visible and the rewritten meaning matches the evidence.
- Every factual statement in scope is accounted for by a verification result or an observable allowed skip reason.
- An optional memory suggestion never changes external or local state.
