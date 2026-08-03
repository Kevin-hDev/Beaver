---
name: gap-analysis
description: Finds consequential gaps in requirements, specifications, plans, stories, or docs. Use to identify missing actors, states, failures, boundaries, data, dependencies, and verification. Not for rewriting, fact checking, audits, or implementation.
---

# Gap Analysis

You scan one current artifact or compare two versions against the same stable completeness model. You report only missing information that can block work, cause rework, or change interpretation.

## Actions

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-frame`](actions/01-frame.md) | You receive an artifact to scan | Artifact type, intended consumer, downstream phase, sources, and boundary |
| [`02-detect`](actions/02-detect.md) | The frame is clear | Deduplicated gaps across the completeness categories with evidence and consequence |
| [`03-compare`](actions/03-compare.md) | A previous gap report or previous artifact version is available | Closed, Still Open, and Newly Introduced gaps matched by stable identity |
| [`04-report`](actions/04-report.md) | Candidate gaps and optional comparison are validated | Ranked gaps, direct questions, comparison status, and coverage |

## Rules

- You remain read-only and keep the report in the conversation unless a file is explicit.
- You evaluate completeness relative to the artifact's intended purpose and downstream consumer.
- You scan actors, states, failures, boundaries, data, dependencies, verification, assumptions, and ambiguities.
- You process gaps in ordered batches of at most 30, preserve a continuation cursor, and continue until every consequential gap in scope is covered.
- You assign every gap a stable identity derived from category, source anchor, and normalized missing decision, never from severity or question wording.
- You classify comparison results as Closed, Still Open, or Newly Introduced when a previous report or artifact version is available.
- You scan an earlier artifact version with the same purpose, consumer, boundary, categories, rubric, and identity rules used for the current version.
- You treat a removed or replaced requirement as intent-uncertain unless the user or an authoritative source confirms that the removal was deliberate.
- You classify `blocker` only when downstream work cannot start or be verified, `major` for likely rework, and `minor` for clarity with no changed decision.
- You cite the relevant section or state that the required section is absent.
- You do not report information already defined by an applicable authoritative source.
- You phrase each gap as missing information, consequence, and the smallest direct question that resolves it.
- You never fill gaps, rewrite the artifact, choose solutions, or create downstream work implicitly.

## Resources

- Read [gap-rubric.md](references/gap-rubric.md) before scanning or assigning severity.
- Read [gap-identity.md](references/gap-identity.md) before assigning keys or comparing with a prior report.
- Copy [gap-report-template.md](assets/gap-report-template.md) only when a report file is requested.
