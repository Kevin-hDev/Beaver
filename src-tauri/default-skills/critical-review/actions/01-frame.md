# 01 - Frame

You establish what the work is meant to achieve and which evidence can fairly challenge it.

## Input

- Accept one proposal, plan, design, specification, decision, answer, implementation approach, completed reasoning, file set, or validated commit range challenged as a whole.
- Accept the agreed reference when one exists.

## Output

- Return the intended outcome, constraints, review boundary, evidence, and unresolved framing questions.

## Process

1. **Identify work.** You name the exact artifact or reasoning being challenged and exclude unrelated surrounding work.
2. **Recover intent.** You derive the desired outcome from explicit requirements, prior decisions, or the user's stated goal.
3. **Separate constraints.** You distinguish mandatory constraints from preferences, assumptions, and proposed implementation choices.
4. **Choose evidence.** You prioritize accepted requirements, current project facts, measured data, and authoritative sources.
5. **Expose ambiguity.** You ask one focused question only when competing interpretations would materially change the verdict.
6. **Set batches.** You process candidate blockers and improvements in ordered batches of at most 15 and alternatives in batches of at most three. You keep a continuation ledger and continue until every supported candidate in scope has been considered.

## Stop conditions

- You return `inconclusive` when no intended outcome can be recovered without inventing one.
- You do not treat the current approach as a requirement merely because it is already detailed.
- You do not inspect unrelated code or sources beyond what the challenge needs.
- You do not turn a file set or commit range into a line-level defect review.

## Test

- The frame separates outcome, mandatory constraints, assumptions, and solution choices.
- Every later finding can be judged against a named reference or an explicitly stated first-principles test.
