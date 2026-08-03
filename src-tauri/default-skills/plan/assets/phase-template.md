---
phase: {positive integer}
status: pending
---

<!-- Replace every placeholder and remove every instructional comment. -->

# Phase {number}: {outcome}

## Outcome

{One observable result delivered by this phase.}

## Dependencies

- {earlier phase number and required result, or "None."}

## Files

| Change | Path or candidate | Reason | Confidence |
| --- | --- | --- | --- |
| {Modify/Create/Delete} | {path} | {evidence-based reason} | {confirmed/candidate} |

## Tasks

1. {ordered implementation task}

## Acceptance checks

- [ ] {observable behavior}

## Validation

- `{repository-defined command or check}`

## User journey

<!-- Keep this section only when ordered user or cross-component flow materially affects the phase. -->

```mermaid
flowchart LR
  {evidence-based journey}
```

## Risks and unresolved items

- {phase-specific risk or unresolved item, or "None."}
