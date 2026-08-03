# 01 - Scan

You read the current project into a silent, bounded snapshot.

## Input

- Accept the current workspace, the user's stated goal, and an optional named plan or task.

## Output

- Return an internal snapshot containing foundation, delivery, health, capability, and session-ledger evidence without printing it to the user.

## Process

1. **Validate scope.** You resolve the current workspace, validate every explicit path or revision, reject traversal, and remain inside the requested project.
2. **Read instructions.** You read the applicable project instructions before you inspect governed files.
3. **Evaluate state.** You read [state-model.md](../references/state-model.md) and classify every applicable check as `met`, `drift`, `missing`, `blocked`, or `unknown`, with direct evidence.
4. **Detect foundations.** You distinguish greenfield projects from established projects and check technical vision, durable project context, and context references only when each foundation applies.
5. **Locate delivery.** You find current specifications, plans, changed code, validation or review evidence, commits, and current-branch review requests. You use the furthest proven stage and never infer completion from filenames alone.
6. **Apply plan status.** You use a readable plan status to refine the stage. You mark an absent or invalid status as uncertainty rather than skipping review or delivery gates.
7. **Detect health signals.** You inspect source-only evidence for missing tests, reported failures or bug markers, and unusually complex code. You exclude templates, fixtures, generated output, dependencies, and installed capability folders.
8. **Resolve capabilities.** You inventory only the skills, tools, connectors, and commands exposed to the active session or documented by the project. You keep unavailable surfaces unknown.
9. **Apply ledger.** You remove a step from the actionable set when current evidence proves it done or the session ledger records it completed, skipped, reviewed, or intentionally left.
10. **Continue batches.** You inspect at most 50 relevant files or 100 direct matches per batch, preserve a cursor, and continue until every check needed for assessment has evidence or a real blocker.
11. **Hold snapshot.** You keep the snapshot internal and pass it to assessment without rendering an interim report.

## Stop conditions

- You stop when the workspace or required source cannot be validated or read.
- You stop before reading a secret, personal file, unrelated global directory, or data outside the requested project.
- You keep a check `unknown` when the available evidence cannot prove its state.

## Test

- The action changes no file and prints no partial snapshot.
- Every classified check has current evidence or an explicit unknown reason.
- A handled ledger step and a review request from another branch never enter the actionable set.
- A batch boundary never becomes a silent end to scanning.
