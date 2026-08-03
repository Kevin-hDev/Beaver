# 01 - Assess Coverage

You identify the smallest set of uncovered behaviors that would materially improve confidence in the requested scope.

## Input

- Accept a feature, module, changed area, or explicit behavior to test.
- Use project instructions, test configuration, neighboring tests, and the relevant contract or implementation.

## Output

- Return a prioritized behavior list in batches of at most 12, with complete requested-scope coverage across continuable batches.
- Return a 0-to-5 need score, existing evidence, missing proof, proposed test level, and observable expectation for each behavior.
- Return an explicit approval gate before any test file changes and a `pending` entry for every declined behavior.

## Process

1. **Validate scope.** You reject invalid paths, preserve unrelated work, and stop if the target cannot be identified.
2. **Read conventions.** You inspect the applicable project rules, test commands, helpers, fixtures, and nearby examples.
3. **Trace behavior.** You follow public inputs to observable outputs and note existing coverage without assuming that file coverage proves behavior.
4. **Collect candidates.** You inspect at most 50 candidate gaps per discovery batch, discard duplicates, private implementation details, and low-value permutations, then continue discovery batches until the requested scope is covered.
5. **Score and prioritize.** You score each retained behavior from 0 (`not needed`) to 5 (`critical core flow`) using user impact, security boundary, regression risk, and absence of existing proof. You order by score and impact, present at most 12 in one approval batch, and continue later batches without dropping the remainder.
6. **Choose level.** You use the lowest test level that exercises the real contract without mocking the subject under test.
7. **Request approval.** You show the current prioritized batch and wait for explicit user approval before writing any test. You do not treat the original request to find gaps as approval of the resulting list.
8. **Record decisions.** You mark approved behaviors for `02-add-tests`. You record each declined behavior as `pending` with the user's reason or `declined by user`.
9. **Continue selection.** After one approved batch is implemented, you present the next batch until every retained behavior is approved, pending, already covered, or blocked.

## Stop conditions

- You stop without writing when the requested behavior is already proven by stable tests.
- You stop when the expected behavior has no reliable source and competing interpretations would change the assertion.
- You stop when isolation from production or real user data cannot be established.
- You do not modify a test file while the current behavior batch awaits approval.

## Test

- Every selected item names one observable expectation and one appropriate test level.
- Every candidate has a justified 0-to-5 need score.
- Each approval batch contains at most 12 behaviors, no duplicate of existing coverage, and the remaining ordered list is preserved for later batches.
