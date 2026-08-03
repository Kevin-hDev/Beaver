# Research method

You use this method for a standalone research request and before every new or materially changed playbook.

## Angles

- **Alternatives:** competing tools, libraries, procedures, or architectures.
- **New methods:** material techniques introduced since the existing playbook or project convention.
- **Coverage gaps:** required subtopics or failure paths the playbook omits.
- **Counter-intuitive wins:** surprising practices that outperform an obvious default, with evidence of the result.
- **Deprecations:** recommendations that are discouraged, removed, unsafe, or abandoned.
- **Operations:** adoption signals, migration costs, compatibility, common failures, and recovery notes.

Cover angles independently in parallel when possible, or sequentially in ordered resumable batches. Give each surviving candidate a separate verification pass so one attractive claim cannot validate the rest. Continue until every defined angle and completion-checklist item clears; do not impose an arbitrary total candidate cap. Do not let the absence of parallel capacity reduce coverage.

## Evidence bar

- Prefer current official documentation, specifications, source repositories, release notes, and original research.
- Record version, release date, or publication date for time-sensitive claims. Mark stale evidence and explain its effect.
- Use community discussions, issue trackers, and adoption metrics as operational signal. Do not let popularity prove correctness, security, or existence.
- Corroborate material tradeoffs. Resolve contradictions or report them as unknown rather than choosing the convenient source.
- Capture practical gotchas, compatibility limits, migration notes, and failure recovery.

## Candidate verification

For every surviving candidate, confirm:

- It exists, is maintained enough for the stated use, and matches the requested scope.
- Its canonical official link and current state are known.
- Its claimed benefit and tradeoffs are supported.
- A real example is available from an official source or safe execution.

Drop a candidate that cannot be confirmed against a primary or official source. Preserve a clearly labeled unknown only when the unknown itself materially affects the decision.

## Reporting

Return alternatives with pros and cons, coverage gaps, counter-intuitive wins, deprecations, unknowns, and a recommendation tied to the refined goal. Sort by value, not novelty. Research remains ephemeral until the user explicitly requests a project playbook write.
