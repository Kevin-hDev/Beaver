# 03 - Compare Alternatives

You compare the current approach with only the alternatives that could materially improve correctness, simplicity, risk, or cost.

## Input

- Accept the current approach, intended outcome, mandatory constraints, and challenge findings.

## Output

- Return the complete batched comparison or state that no better alternative is supported.

## Process

1. **Generate sparingly.** You include an alternative only when it changes a meaningful decision rather than renaming the same approach. You keep a continuation ledger when more than three candidates remain.
2. **Prefer subtraction.** You test whether removing a component, state, abstraction, dependency, or process step still achieves the outcome.
3. **Check viability.** You reject alternatives that violate mandatory constraints or depend on unavailable capabilities.
4. **Compare consistently.** You compare at most three alternatives per batch using the same criteria for every option: outcome coverage, risk, complexity, reversibility, evidence, and cost. You continue with the next batch until every decision-relevant candidate is covered.
5. **Expose tradeoffs.** You state what each option improves, worsens, and leaves unresolved.
6. **Choose only when supported.** You recommend retaining or replacing the current approach only when the comparison provides enough evidence.

## Stop conditions

- You do not process more than three alternatives in one batch or silently discard a remaining decision-relevant alternative.
- You do not prefer novelty, elegance, or fewer files when it worsens the intended outcome.
- You do not recommend a replacement whose migration or failure cost was not considered.

## Test

- Every compared option is viable under the mandatory constraints.
- A recommendation cites the criteria it wins and the tradeoffs it accepts.
