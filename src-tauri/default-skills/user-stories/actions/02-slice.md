# 02 - Slice

You decompose an epic into independent vertical outcomes that each deliver recognizable value.

## Input

- Accept the approved scope and epic-or-story classification.

## Output

- Return ordered candidate-story batches with user value and dependencies.

## Process

1. **Preserve one story.** You emit one candidate when the scope contains one independently valuable outcome.
2. **Slice by outcome.** You divide an epic by workflow step, user goal, business rule, scenario, or progressive capability.
3. **Avoid layers.** You reject candidates that deliver only UI, API, storage, migration, or infrastructure without user value.
4. **Test independence.** You reshape or name real dependencies when a candidate cannot deliver value alone.
5. **Control size.** You split candidates that cannot fit one team iteration or cannot be estimated without internal tasks.
6. **Batch the backlog.** You keep at most 20 candidates in one batch, carry dependency and scope context forward, and continue in order until every accepted outcome has one candidate.
7. **Confirm.** You show the complete candidate set across all batches and wait for explicit user approval before drafting.

## Stop conditions

- You stop when slicing would invent product behavior or erase a mandatory dependency.
- You never call technical tasks user stories merely to make them fit the template.

## Test

- Every candidate has a user-perceivable outcome and can be accepted independently or names a real dependency.
- Every batch contains no more than 20 stories, the union covers every accepted outcome, and no batch contains a technical-layer slice.
- The user approves the complete candidate set before drafting starts.
