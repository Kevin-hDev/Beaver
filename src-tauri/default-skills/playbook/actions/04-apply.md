# 04 - Apply a playbook

You analyse, confirm, execute, verify, and report an existing playbook as a resumable checklist.

## Input

- Accept a playbook selected by the latest list number, exact slug, title, path, or unambiguous topic.
- Accept an optional subset of steps, stated exclusions, and an optional prior execution checkpoint.

## Output

- Return a preflight analysis of the outcome, prerequisites, risks, and every step classified as `agent-doable`, `human-only`, `unsupported`, or `out-of-scope`.
- Return the confirmed execution ledger, verification evidence, remaining human steps, resumable checkpoint, and `complete`, `partial`, or `blocked` status.

## Process

1. **Locate.** You read [locations.md](../references/locations.md) and resolve the playbook. You rerun the list action and ask for a new number when a numeric choice has no current numbered mapping.
2. **Read and preflight.** You read the full playbook, inspect relevant project state, identify prerequisites and reversible boundaries, and reject path escapes, secret exposure, unbounded work, or effects outside the requested project scope.
3. **Classify every step.** You mark a step `agent-doable` only when available tools can perform and verify it safely. You mark interactive UI, credentials, external-account decisions, unsupported operations, and system-wide changes as `human-only` or `unsupported`.
4. **Show exact effects.** You state what the playbook achieves, list files, commands, services, dependencies, and external effects each agent-doable step would change, and identify all human-only work.
5. **Ask before mutation.** You ask the user to choose all agent-doable steps, a subset, or report-only. You make no state change before the answer. Before each selected state-changing step, including file, process, dependency, service, destructive, irreversible, or external effects, you obtain explicit confirmation for its exact target and effect.
6. **Create the ledger.** You number selected steps and record status, evidence, and next action. You keep the ledger in the conversation unless the user explicitly selects a safe project-local checkpoint destination.
7. **Execute safely.** You run only confirmed agent-doable steps in order, validate external input, use direct argument lists for system commands, treat errors as blocking for the affected step, and never log or print secrets. You process long work in bounded resumable batches without imposing a silent total limit.
8. **Resume honestly.** After interruption, you re-read project state and prior evidence before continuing. You do not assume an `in-progress` step completed and do not repeat a non-idempotent action without proof that retry is safe.
9. **Verify.** You run every applicable `## Verify` check plus step-specific observable checks. You record actual results and leave unmet checks `blocked` or `partial`.
10. **Report.** You summarize verified changes, skipped and blocked steps, untouched human-only instructions, verification results, and the exact checkpoint from which work can resume.

## Stop conditions

- You stop before mutation until the user selects steps and confirms every required state-changing effect.
- You stop the affected branch when a prerequisite, safety check, command, write, or verification fails; you do not fail open.
- You stop and return `partial` or `blocked` when a human-only step gates later work, the playbook conflicts with current project state, or requested effects exceed the playbook.

## Test

- Confirm that every playbook step was classified before any change and that the user selected the execution scope.
- Confirm that every state-changing step has explicit confirmation, ledger status, and evidence.
- Confirm that human-only and unsupported steps were never executed.
- Confirm that verification ran and incomplete work includes a non-fabricated resumable checkpoint.
