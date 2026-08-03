# 02 - Challenge

You test the work's logic, completeness, necessity, feasibility, and risk without rewarding complexity or contrarianism.

## Input

- Accept the framed work, outcome, constraints, assumptions, and evidence.

## Output

- Return strengths, blockers, improvements, and unresolved claims linked to the outcome.

## Process

1. **Rebuild the argument.** You express how the proposed work is expected to produce the intended outcome.
2. **Test assumptions.** You look for unsupported, circular, stale, contradictory, or unnecessary assumptions.
3. **Test completeness.** You check required outcomes, failure paths, operational constraints, users, data, dependencies, and validation evidence relevant to the scope.
4. **Test feasibility.** You identify steps that depend on unavailable capability, incompatible constraints, hidden coordination, or unverifiable success.
5. **Test necessity.** You look for duplicated mechanisms, premature abstraction, avoidable state, and work that does not contribute to the intended outcome.
6. **Preserve strengths.** You name decisions that are correct, well-supported, or appropriately simple.
7. **Classify.** You separate blockers, improvements, and unresolved evidence according to the shared rubric.

## Stop conditions

- You do not report a preference as a blocker.
- You do not repeat the same cause across several symptoms.
- You do not present an unverified factual claim as a confirmed flaw.
- You do not rewrite or implement the work during the challenge.

## Test

- Every blocker states the failed outcome or constraint, supporting evidence, and consequence.
- Every improvement remains optional and every unresolved item names the missing evidence.
