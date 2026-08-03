# 04 - Compose

You turn the confirmed scope and repository evidence into a complete candidate plan.

## Input

- Use the gathered scope, exploration output, and approved UI sketch when one applies.

## Output

- Return a candidate plan containing phases, file impact, tasks, acceptance checks, validation, risks, and unresolved items, ready for review.

## Process

1. **Read.** You inspect [phase-design.md](../references/phase-design.md).
2. **Slice.** You group the work into the fewest coherent phases that can each be implemented and verified without hidden prerequisites. You process at most 20 phases per numbered batch and 50 tasks per phase, preserve global phase numbering, and continue later batches until every phase is specified.
3. **Specify.** You give every phase an outcome, bounded file impact, ordered tasks, observable acceptance checks, and exact validation gates supported by the repository. When a phase changes a user or cross-component sequence whose ordering matters, you include a concise Mermaid user-journey diagram; you omit diagrams that do not clarify a real flow decision.
4. **Sequence.** You place shared contracts and data changes before their consumers. You place cleanup only where the requested work makes it necessary.
5. **Reconcile.** You confirm that the phases cover every required behavior, projected file, migration, and test without adding unrelated work.
6. **Expose uncertainty.** You list risks, assumptions, and unresolved decisions. You do not bury them inside tasks.
7. **Choose delivery.** You show the complete candidate plan in the conversation by default. When the user requests one plan file, you copy [plan-template.md](../assets/plan-template.md). When the user requests a source-style bundle or the established project workflow requires one, you copy [plan-index-template.md](../assets/plan-index-template.md) to `plan.md` and [phase-template.md](../assets/phase-template.md) to one `phase-<number>.md` file per phase.
8. **Persist safely.** You canonicalize the requested destination, reject traversal, render every requested artifact, remove all placeholders, verify that `plan.md` links every phase file exactly once, stage every file beside its destination, and replace each file atomically. You fail closed before replacement when any artifact is invalid and never overwrite unrelated files.
9. **Mark state.** You keep every delivered artifact in candidate state until explicit approval.

## Stop conditions

- You stop when a phase depends on an unresolved product or architecture decision.
- You stop when the plan cannot name an observable completion check.
- You do not implement any task or perform Git operations.

## Test

- The union of the phases covers the complete confirmed scope and nothing unrelated.
- Every phase appears exactly once across all numbered batches, including plans longer than 20 phases.
- Every acceptance check describes observable behavior, and every validation gate is executable in the project.
- A written bundle contains only `plan.md` plus one deterministically named file per phase, every index link resolves, and no placeholder remains.
- The result remains a candidate until `05-review` records explicit approval.
