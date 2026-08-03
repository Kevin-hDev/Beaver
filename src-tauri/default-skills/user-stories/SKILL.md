---
name: user-stories
description: Creates independent, valuable, testable user stories with acceptance criteria, dependencies, effort, and priority. Use to split a product outcome or epic into a bounded backlog. Not for PRDs, technical plans, implementation, or generic tickets.
---

# User Stories

You create a small ordered backlog of vertical user outcomes that can be understood, estimated, and verified independently.

## Actions

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-scope`](actions/01-scope.md) | You receive an epic or story request | Accepted outcome, users, boundaries, constraints, and epic-or-story classification |
| [`02-slice`](actions/02-slice.md) | The scope contains one or more outcomes | Ordered batches of independent vertical candidate stories |
| [`03-draft`](actions/03-draft.md) | Candidate slices are stable | INVEST stories with acceptance criteria and functional completion conditions |
| [`04-prioritize`](actions/04-prioritize.md) | Stories are testable and estimable | Dependencies, relative effort confidence, value, risk, and strict order |
| [`05-deliver`](actions/05-deliver.md) | The backlog passes readiness | Conversation backlog, requested file, or explicitly requested tracker batches |

## Rules

- You slice by user-visible outcome, not frontend, backend, database, or infrastructure layers.
- You enforce Independent, Negotiable, Valuable, Estimable, Small, and Testable for every story.
- You process stories in ordered batches of at most 20, carry dependencies and ranking context forward, and continue until every accepted outcome is covered.
- You write observable acceptance criteria with nominal and relevant failure or boundary behavior.
- You keep completion conditions functional and never list code review, coverage, deployment, or other technical delivery steps as user value.
- You use existing team estimation policy; otherwise you give relative size with low, medium, or high confidence and never false precision.
- You rank by value, dependency, risk reduction, and effort, and explain any dependency override.
- You keep a story not ready while an unaccepted assumption can change its scope, acceptance, dependency, or user value.
- You keep the backlog in the conversation unless a file or tracker write is explicit.
- You create tracker stories only when explicitly requested, a connector exists, every story is ready, and each external write batch contains at most 20 items.

## Resources

- Read [story-rubric.md](references/story-rubric.md) before slicing, checking readiness, estimating, or prioritizing.
- Copy [user-story-template.md](assets/user-story-template.md) only for a requested file or tracker body.
