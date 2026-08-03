# 03 - Validate

You score a specification against the shared rubric without modifying it.

## Input

- Accept a specification as text or a validated path.

## Output

- Return `valid`, `draft-with-gaps`, or `invalid`.
- Return a score and one finding per failed criterion.
- Return precise questions for remaining gaps.

## Process

1. **Validate the source.** You reject empty input, `..` in a supplied path, a path outside the authorized working area, or an unreadable file. You read inline input in ordered chunks of at most 100,000 characters and files in ordered chunks of at most 256 KiB until the complete source is covered.
2. **Read the rubric.** You read [validation-rubric.md](../references/validation-rubric.md).
3. **Check hard failures.** You mark the result `invalid` when a required section is absent, the target contains multiple unrelated goals, a completion condition is not observable, or the draft introduces an unrequired implementation choice.
4. **Score.** You inspect at most 100 requirements, constraints, and completion conditions per batch and continue until coverage is complete. You score each criterion using the rubric. You do not award credit for vague promises or placeholders.
5. **Classify.** You return `valid` only when every required criterion is fulfilled, no hard failure applies, and the score reaches 90. You return `draft-with-gaps` when the structure is sound but explicit `TBD` questions remain. You return `invalid` for structural or intent failures.
6. **Report.** You cite the affected section, explain the gap in one sentence, and ask one precise question for each unresolved required decision.

## Stop conditions

- You stop without a verdict when the source cannot be read reliably.
- You do not modify the specification during validation.

## Test

- The verdict follows the hard failures and score threshold in the rubric.
- Every failed required criterion produces a concrete finding.
- Validation causes no file or external-system change.
