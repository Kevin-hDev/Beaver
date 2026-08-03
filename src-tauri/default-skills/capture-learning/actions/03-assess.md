# 03 - Assess

You score durable candidates, reconcile existing coverage, and recommend the narrowest valid destination.

## Input

- Use the complete candidate batches and validated project root.
- Accept existing project-local memory, ADR, rule, skill, and context conventions discovered from current files.

## Output

- Return each candidate with a 0–10 score, recommendation, `new`, `covered`, `updates`, or `supersedes` reconciliation, destination options, and evidence.
- Return unresolved convention or generator-availability questions without writing or handing off anything.

## Process

1. **Read the contracts.** You read [assessment.md](../references/assessment.md) and [destinations.md](../references/destinations.md).
2. **Resolve conventions.** You inspect current project instructions, indexes, memory files, ADRs, rules, and local skills. You process at most 100 destination files per batch and continue until the relevant existing coverage is checked.
3. **Score candidates.** You score durability, reuse, project fit, and forgetting risk on the shared 0–10 scale and explain the result without false precision.
4. **Reconcile coverage.** You classify each candidate exactly once as `new`, `covered`, `updates`, or `supersedes`. You cite the existing entry for every non-new result.
5. **Protect supersession.** You identify the older decision or rule and the required reverse link. You never treat disappearance or contradiction as proof of intentional supersession.
6. **Choose the smallest destination.** You recommend memory, ADR, rule, or skill according to the destination contract. You preserve the user's right to choose another valid project-local destination.
7. **Check delivery capability.** You identify whether memory or ADR conventions permit a direct project write and whether an appropriate project-memory, rule, or skill generator is actually available. When the project memory bank is missing, you record the exact bank-setup handoff that would be required before reassessment. You record unavailable handoffs explicitly.
8. **Present the assessment.** You show the complete scored and reconciled recommendations across all batches before approval.

## Stop conditions

- You stop and ask when the project memory bank, ADR location, rule convention, skill convention, or context markers are missing or ambiguous. For a missing memory bank, you offer an explicit handoff to an available project-memory capability and wait for approval; you never treat that handoff as delivery of the lesson.
- You do not scaffold a missing memory bank, invent a taxonomy, or inspect personal or global memory.
- You do not write, hand off, or imply approval during assessment.

## Test

- Every candidate has one supported score, reconciliation class, smallest valid destination, and delivery capability state.
- Every `covered`, `updates`, or `supersedes` result cites existing project content.
- Every supersession proposal identifies both required ADR links when ADRs are involved.
