---
name: fact-check
description: Verifies claims against current authoritative project or external sources and returns corrected prose with uncertainty visible. Use to fact-check or add citations. Not for opinions, code review, audits, requirement gaps, or open-ended research.
---

# Fact Check

You preserve the author's intended meaning while verifying each material or nontrivial factual claim, correcting false statements, and citing the strongest available evidence.

## Actions

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-extract`](actions/01-extract.md) | You receive text or named claims to verify | Bounded atomic-claim batches and an observable skip ledger |
| [`02-verify`](actions/02-verify.md) | Claims are precise enough to test | Supported, contradicted, conflicted, or unresolved claims with sources |
| [`03-report`](actions/03-report.md) | Verification is complete | Corrected cited prose, conflicts, unresolved claims, and source list |

## Rules

- You verify the exact claim stated and do not replace it with a more defensible interpretation.
- You extract every material or nontrivial factual claim in the requested scope, split compound claims into atomic statements, and process them in ordered batches of at most 30 until none remain.
- Unless the user requests exhaustive checking, you may skip a fact only when it is trivial, directly self-evident from the supplied text, or irrelevant to the conclusion. You record the statement and one concrete skip reason.
- You never label a disputed, uncertain, cited, numeric, time-sensitive, technical, legal, or conclusion-bearing claim trivial merely because evidence is difficult or unavailable.
- When the user requests exhaustive checking, you verify every factual claim in scope, including trivial and background facts, while continuing to exclude pure non-claims.
- You skip opinions, preferences, predictions, hypotheticals, and the author's stated intent unless they contain a factual assertion.
- You verify project facts from applicable project sources and code; you do not use the web to override current repository truth.
- You verify external unstable facts with current authoritative primary sources and browse when they may have changed.
- You prefer official documentation, specifications, registries, public records, and primary research over summaries.
- You never cite a search result, unsupported snippet, generated answer, or source that does not directly support the claim.
- You report source conflicts without choosing a winner unless a clear authority or newer applicable source resolves them.
- You mark unresolved claims and never silently delete or present them as fact.
- You preserve citations already supported, remove unsupported citations, and respect quotation and source-use limits.
- You persist nothing and never cache verified facts implicitly.
- You may propose an optional memory suggestion for a stable, reusable verified fact, but you never persist it or invoke another capability without an explicit later request.

## Resources

- Read [source-rubric.md](references/source-rubric.md) before choosing sources or resolving conflicts.
- Copy [fact-check-report-template.md](assets/fact-check-report-template.md) only when a report file is requested.
