# 02 - Add Tests

You add focused tests for the selected behaviors and distinguish a test defect from a confirmed product defect.

## Input

- Accept an explicitly approved behavior batch with observable expectations.
- Use the existing framework, command, helpers, fixtures, and isolation mechanism.

## Output

- Return a passing focused test for every approved behavior or a preserved failing test with evidence that the product violates the expected contract.
- Return the exact focused command and its result.
- Return every declined behavior as `pending` with a reason.

## Process

1. **Establish baseline.** You run the smallest relevant existing test command before editing and record unrelated pre-existing failures.
2. **Create data.** You use minimal deterministic fixtures, bound generated cases to 100 per group, and prevent contact with production systems.
3. **Confirm approval.** You write only behaviors in the explicitly approved batch. You carry declined items forward as `pending` and never generate their tests.
4. **Write one behavior.** You match neighboring structure and assert observable input, output, state, protocol exchange, or error behavior.
5. **Run focused test.** You execute the narrowest command that proves the new test.
6. **Classify failure.** You compare the authoritative contract, the test, and actual behavior before changing anything.
7. **Repair in batches.** You make at most three test-only repair attempts per batch when setup or expectation is demonstrably wrong or fragile. If the batch fails without proving a product defect or blocker, you preserve the attempts, revise the next batch's hypotheses, and continue.
8. **Preserve product defect.** You keep the failing test unchanged when it correctly exposes a product defect, then stop before production changes.
9. **Check surrounding suite.** You run the relevant wider suite after the focused test passes, unless a pre-existing blocker makes the result unreliable.
10. **Continue behaviors.** You repeat for every approved behavior in the batch, then return to the next approval batch until the complete selected scope is accounted for.

## Stop conditions

- You stop when the expected behavior is ambiguous or contradicted by equally authoritative sources.
- You stop only for ambiguity, a confirmed product defect, unsafe isolation, or another real blocker. One three-attempt repair batch alone is not terminal.
- You stop when isolation, cleanup, or deterministic execution cannot be guaranteed.
- You stop before changing production behavior, installing a dependency, or silencing a valid failure.

## Test

- Each added test fails for the intended behavioral difference and passes when that behavior is present.
- No new test reaches production, contains a real secret, or depends on fixed sleeps without a project-provided reason.
- Every approved behavior has one result, and every declined behavior remains visible as `pending`.
