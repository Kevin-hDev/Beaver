---
name: product-requirements
description: Defines or refines a PRD from a product need, PRD, or supplied user stories. Covers users, outcomes, scope, constraints, measures, dependencies, and questions. Not for technical specs, plans, backlog creation or management, gap scans, or code.
---

# Product Requirements

You produce a concise solution-agnostic product contract that explains what must change, for whom, why it matters, and how success is observed.

## Actions

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-frame`](actions/01-frame.md) | You receive a product need, PRD, or existing user stories | Problem, users, source-story traceability, desired outcomes, and decision gaps |
| [`02-define`](actions/02-define.md) | The product problem is clear enough | Goals, non-goals, scope, requirements, user stories, acceptance criteria, constraints, dependencies, measures, and questions |
| [`03-draft`](actions/03-draft.md) | Required product decisions are available | A complete solution-agnostic PRD |
| [`04-finalize`](actions/04-finalize.md) | The PRD is internally consistent | Final conversation result or explicitly requested file |

## Rules

- You describe the problem, users, outcomes, boundaries, and evidence; you do not select architecture, technology, schemas, APIs, or files.
- You distinguish facts, user statements, assumptions, and open questions.
- You accept existing user stories supplied as text, identifiers, or validated tracker URLs as source evidence for reconstructing or refining a PRD.
- You consume supplied stories without creating, editing, estimating, prioritizing, transitioning, or otherwise managing their backlog.
- You process goals, non-goals, requirements, user stories, acceptance criteria, dependencies, and open questions in bounded ordered batches, continue until the complete accepted product scope is covered, and never omit overflow silently.
- You include concise user stories and observable acceptance criteria in the PRD while keeping detailed backlog management and implementation planning outside this skill.
- You make goals and success measures observable without inventing baselines or target numbers.
- You include failure, accessibility, privacy, compliance, localization, platform, and operational constraints only when relevant.
- You preserve unresolved decisions as explicit questions instead of silently choosing.
- You keep the PRD in the conversation unless the user requests a file or supplies an established destination.
- You never create tickets, plans, code, branches, commits, or external documents implicitly.

## Resources

- Read [product-rubric.md](references/product-rubric.md) while separating requirements from implementation and checking completeness.
- Copy [product-requirements-template.md](assets/product-requirements-template.md) only when a file is requested.
