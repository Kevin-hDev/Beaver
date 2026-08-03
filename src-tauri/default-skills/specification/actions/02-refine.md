# 02 - Refine

You revise an existing draft only where supplied findings require a change.

## Input

- Accept an existing specification as text or a validated path.
- Accept review findings or requested corrections in bounded batches.

## Output

- Return the revised specification at the supplied destination when applicable.
- Return one resolution or `TBD` mapping per finding.
- Return a handoff to `03-validate`.

## Process

1. **Validate.** You reject an empty specification, an empty findings set, a path containing `..`, or a path outside the authorized working area. You read long specifications in bounded chunks and process findings in ordered batches of at most 100 until every finding is accounted for.
2. **Protect locks.** You stop without modification when the specification contains `status: locked` or the user identifies it as locked.
3. **Map.** You pair every finding with Target, Hard constraints, Non-goals, Done-when, Stakeholders, or Context. You flag a finding that does not belong in a specification.
4. **Revise narrowly.** You change only the affected sections. You preserve the remaining wording, order, and confirmed intent.
5. **Handle gaps.** You write `TBD: <precise question>` when a finding exposes a decision that the source cannot resolve.
6. **Check completeness.** You ensure every required section remains present and every finding has a recorded outcome.
7. **Persist safely.** When the input is a file, you verify that it has not changed since you read it, then replace it atomically at the same validated path. You stop on any write failure.
8. **Validate.** You read `03-validate` and check the revised draft.

## Stop conditions

- You stop without changes when the specification is locked.
- You stop when findings contradict one another and the intended precedence is unknown.
- You stop when the source changes during refinement or a write fails.

## Test

- Every finding maps to a targeted change, an explicit rejection, or a precise `TBD` question.
- Unaffected sections remain unchanged.
- Required sections remain present and ordered.
- A locked specification remains byte-for-byte unchanged.
