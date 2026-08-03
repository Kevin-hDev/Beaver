# 02 - Assess

You turn the current snapshot into one decision.

## Input

- Accept the complete snapshot from `01-scan` and the user's stated goal.

## Output

- Return one ranked decision, its evidence, one primary available action or functional gap, and optional secondary choices.

## Process

1. **Classify state.** You use [ranking.md](../references/ranking.md) to classify the project as `greenfield`, `existing`, `drift`, `midwork`, `blocked`, or `idle`.
2. **Rank foundations.** You choose the earliest missing or drifting applicable foundation. You suppress delivery, health, and idle options until the required foundations are settled.
3. **Rank delivery.** When foundations are clear, you choose the earliest unmet delivery stage supported by current evidence and the user's goal.
4. **Respect blockers.** You surface a blocked or invalid plan before any normal next stage. You never route around a failed required validation or review.
5. **Rank health.** When no delivery stage is active, you select a fired health signal only when an available capability can address it or a functional gap must be reported.
6. **Build idle choices.** When the project is idle, you use [workflow.md](../references/workflow.md) to offer start work, improve health, customize project automation, and explore. You remove an empty category.
7. **Resolve exact actions.** You match every executable choice to an exact capability exposed to the active session. You describe missing functionality without inventing an identifier.
8. **Set behavior.** You classify an available action as `complete-now`, `interactive-handoff`, or `instruction-only` from its real contract.
9. **Keep one decision.** You select one primary action and at most three secondary choices. You retain the full snapshot only for details.

## Stop conditions

- You stop when contradictory evidence could change the primary action and ask one focused question.
- You do not select an unavailable capability or a downstream action while a required upstream state is missing.
- You return a functional gap when no exposed capability can perform the required action.

## Test

- One and only one primary action is selected.
- Foundations outrank delivery, delivery outranks health, and health outranks idle choices.
- Every executable choice maps to an exact exposed capability.
- An invalid or blocked plan never becomes a normal implementation recommendation.
