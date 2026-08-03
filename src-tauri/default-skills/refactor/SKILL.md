---
name: refactor
description: Refactors code through cleanup, performance, security, or architecture with evidence and regression proof. Use to optimize, harden, restructure, remove code, or apply audit findings. Not for read-only diagnosis, test-only work, upgrades, or redesign.
---

# Refactor

You improve existing code through one explicit axis while keeping every claim tied to a diff, measurement, or executed check.

## Workflow

```mermaid
flowchart TD
    Request([Refactor request]) --> Select{Axis specified?}
    Select -->|Performance| Performance["01 - Performance"]
    Select -->|Security| Security["02 - Security"]
    Select -->|Cleanup or removal| Cleanup["03 - Cleanup"]
    Select -->|Architecture| Architecture["04 - Architecture"]
    Select -->|No| Clarify["Ask: all applicable or one axis?"]
    Clarify --> Axis[Run selected axes separately]
    Performance --> Verify{Required evidence passes?}
    Security --> Verify
    Cleanup --> Verify
    Architecture --> Verify
    Verify -->|Yes| Done([Report verified changes])
    Verify -->|No| Incomplete([Report incomplete or blocked])
```

## Actions

Read only the action for the selected axis before you edit code. When the user chooses all applicable axes, complete and verify each applicable action separately.

| Action | Use it when | Required proof |
| --- | --- | --- |
| [`01-performance`](actions/01-performance.md) | You must reduce runtime, allocation, redundant work, blocking, N+1 access, or unnecessary I/O | Comparable baseline and post-change evidence |
| [`02-security`](actions/02-security.md) | You must fix input, access-control, secret, injection, or other security weaknesses | One regression test per fix and disclosed behavior changes |
| [`03-cleanup`](actions/03-cleanup.md) | You must clarify, deduplicate, simplify, or delete code | Preserved public behavior and concrete diff mapping |
| [`04-architecture`](actions/04-architecture.md) | You must restore boundaries, layering, isolation, or dependency direction | Per-step verification and a clean boundary graph |

## Rules

- You run the named axis. For an unscoped refactor request, you ask once whether to run all applicable axes or one selected axis; you never silently choose. You treat an explicit delete or removal request as cleanup without asking about axes.
- You accept a pasted or readable audit report as the fix list for the matching axis and skip broad discovery. You map `code-quality` findings to cleanup and the other axes by name. You verify that each finding still matches current code before editing.
- You establish the pre-change behavior, check result, structural state, or measurement required by the selected action. You report `baseline unavailable` when comparable evidence cannot be obtained and never invent a gain or preserved behavior.
- You preserve observable behavior for cleanup, performance, and architecture. You allow a security fix to reject behavior that was previously unsafe, and you disclose that intentional change.
- You keep changes minimal and inside the selected axis. You do not upgrade dependencies, redesign interfaces, add unrelated features, or combine opportunistic cleanup with another axis.
- You add new tests only as regression coverage for security fixes. For other axes, you run existing tests, type checks, representative side-by-side checks, benchmarks, or boundary checks without expanding test scope.
- You process findings and changes in bounded coherent batches, retain evidence between batches, and continue until the selected scope is complete or a real blocker remains.
- You preserve unrelated work, validate user-provided paths and globs before use, keep secrets out of output and logs, and fail closed when required verification fails.
- You use the severity and evidence rules in [evidence-contract.md](references/evidence-contract.md) and report every applied, deferred, stale, or blocked finding honestly.

## Resources

- Read [evidence-contract.md](references/evidence-contract.md) for audit mapping, severity, baseline, verification, and result requirements.
- Read [security-checklist.md](references/security-checklist.md) when running the security action.
- Read [architecture-boundaries.md](references/architecture-boundaries.md) when running the architecture action.
