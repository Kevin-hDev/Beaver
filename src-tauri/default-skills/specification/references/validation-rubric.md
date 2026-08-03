# Specification Validation Rubric

## Criteria

| Criterion | Required | Weight | Fulfilled when |
| --- | --- | ---: | --- |
| Target | Yes | 25 | One clear, verifiable primary outcome is stated in one sentence. |
| Hard constraints | Yes | 20 | Every non-negotiable constraint is explicit, sourced, concrete, and testable. |
| Non-goals | Yes | 20 | The reader can identify what must not be built in this scope. |
| Done-when | Yes | 25 | Every item describes an observable outcome or state. |
| Stakeholders | No | 5 | Decider, owner, and consumer are named when the work requires them. |
| Context | No | 5 | Relevant background, assumptions, and authoritative links are concise. |

## Hard failures

Return `invalid` regardless of score when any condition applies:

- A required section is absent.
- Target contains multiple unrelated primary outcomes.
- A Done-when item cannot be observed or verified.
- The draft invents an implementation choice that is not a required constraint.
- A statement presents an unresolved assumption as confirmed fact.

## Verdicts

- Return `valid` when every required criterion is fulfilled, no hard failure applies, and the score is at least 90.
- Return `draft-with-gaps` when the structure is valid but one or more explicit `TBD` questions remain.
- Return `invalid` when a hard failure applies or the intent cannot be verified.
