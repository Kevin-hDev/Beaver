# Story Rubric

Use this rubric while slicing, drafting, checking readiness, estimating, prioritizing, and reconciling batches.

## INVEST

- **Independent:** You make the story deliver value without another story in the same batch. When a real dependency cannot be removed, you name it and preserve dependency order.
- **Negotiable:** You state the user need and outcome without freezing architecture, files, APIs, schemas, libraries, or delivery tasks.
- **Valuable:** You name an outcome that a user or stakeholder can perceive, confirm, or rely on.
- **Estimable:** You keep the scope and unknowns clear enough for the team to size the story honestly.
- **Small:** You keep one independently valuable outcome that fits one normal iteration. You return larger work to slicing.
- **Testable:** You give the story observable acceptance criteria that produce an objective pass or fail.

## Readiness gate

Mark a story ready only when every condition holds:

- You provide testable acceptance criteria.
- You name dependencies or confirm that none remain.
- You give a supported size and confidence.
- You give an impact rating with evidence.
- You resolve every blocking product question.
- You resolve or reject every assumption that can change scope, acceptance, dependency, or user value.

Mark the story `not ready` and retain its direct blocking question when any condition fails. Do not call a story ready under an unaccepted assumption.

## Acceptance criteria

Cover the nominal user outcome and every relevant supported empty, error, permission, limit, accessibility, recovery, or compatibility behavior. Use Given/When/Then when it makes the state transition clearer. Do not invent a product rule merely to fill an edge-case checklist.

## Functional completion

Write two to five observable user-facing conditions that show the story's value is available. State what the user can do, see, receive, or avoid. Exclude code review, coverage, deployment, file changes, internal tasks, and other technical delivery steps.

## Estimation

Use the team's documented points or sizing policy when it exists. Without one, use small, medium, or large with low, medium, or high confidence. Never invent person-days, deadlines, velocity, or exact business-value scores. Return a story to slicing when its size or uncertainty prevents a defensible estimate.

## Impact

- **Isolated:** You change an additive or contained behavior with no supported effect on existing shared flows.
- **Shared behavior:** You change behavior, contracts, or components used by existing flows and require broader regression checks.
- **Critical path:** You affect authentication, authorization, payments, data integrity, destructive behavior, availability, or another supported high-consequence path.

Give one evidence-based rationale for every impact rating. Keep impact separate from effort.

## Prioritization

Rank the complete backlog by user value and risk reduction relative to effort. Apply real dependency ordering after value ranking and explain every override. Give every story one unique global rank only after all batches are assessed.

## Batch reconciliation

Process at most 20 stories per batch. Carry accepted scope, dependency edges, estimation policy, and value context into every later batch. Before delivery, verify that the union of batches covers every accepted outcome exactly once and that global ranking preserves cross-batch dependencies.
