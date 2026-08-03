# 01 - Scope

You frame a new workflow skill and obtain confirmation before any file changes.

## Input

- Accept the user's goal, examples, constraints, expected result, and optional destination.
- Use the available skill catalog, neighboring bundles, and project-local conventions as evidence.

## Output

- Return a confirmed frame containing purpose, name, trigger examples, non-trigger examples, interaction mode, planned checkpoints, workflow boundary, destination, and overwrite boundary.

## Process

1. **Clarify the result.** You reduce the request to one reusable capability and list the observable result it must produce.
2. **Collect concrete cases.** You obtain at least three realistic trigger requests and two nearby requests that must not trigger the skill. You ask one focused question at a time only when the answer changes the design.
3. **Choose the interaction mode.** You use `auto` when the workflow should continue without planned user checkpoints and `interactive` when it must pause at named decisions. You omit the mode only when the workflow has no meaningful choice between them.
4. **Resolve the destination.** You read [destination-resolution.md](../references/destination-resolution.md), inspect project-local evidence, and require the user to confirm the exact target root when no binding convention exists.
5. **Choose the name.** You read [naming.md](../references/naming.md), propose a kebab-case name, and compare its description and cases with neighboring skills.
6. **Define the boundary.** You state what the skill owns, what remains outside it, which existing files may change, and which capabilities must be preserved.
7. **Confirm the frame.** You present the complete frame and wait for explicit confirmation before handing it to planning.

## Stop conditions

- You stop before writing when the purpose, destination, name, examples, required interaction mode, boundary, or overwrite scope is missing or ambiguous.
- You stop when the request is a one-shot command, a continuously enforced rule, a CLI/API integration, or application code rather than a workflow skill.
- You do not infer a destination from another CLI, a loaded skill's source, or a familiar directory layout.

## Test

- You verify that no file changed during scoping.
- You verify that the frame contains at least three trigger cases, two non-trigger cases, an interaction-mode decision, one exact destination, and one explicit overwrite boundary.
- You verify that every material overlap is surfaced without merging distinct methods or outputs automatically.
