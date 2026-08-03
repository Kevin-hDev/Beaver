# 01 - Collect

You resolve the exact review target and gather enough context to assess it without changing state.

## Input

- Accept a working-tree diff, staged diff, commit, revision range, pull-request change set, or explicit changed paths.

## Output

- Return the target identity, selected axes, base and head when applicable, merge base for branch reviews, bounded file batches, applicable project rules, intended behavior sources, and unavailable evidence.

## Process

1. **Validate.** You validate the repository root, revision syntax, paths, and remote identifier before you use them. You reject traversal and ambiguous revisions.
2. **Select axes.** You select all three axes by default. When the user explicitly names `code`, `functional`, or `relevancy`, you select only that axis and record the other two as not run. You ask one focused question when the requested axis is ambiguous.
3. **Resolve explicit target.** You use the exact target supplied by the user, including a standalone changed-file snapshot without Git history.
4. **Resolve default target.** When no target is supplied, you resolve the repository's confirmed default branch from local or remote metadata without assuming its name. On a non-default branch, you compute the merge base between the default branch and `HEAD`, review every committed change from that merge base through `HEAD`, and include staged and working-tree overlays without duplication. On the default branch, you review staged and working-tree changes against `HEAD`.
5. **Bound input.** You apply the file, byte, and batch limits from [evidence-rules.md](../references/evidence-rules.md). You partition large inputs and preserve full coverage.
6. **Read rules.** You load the project instructions that apply to the changed files.
7. **Find intent.** You resolve the plan, specification, issue, pull-request description, or user request that defines the intended behavior. When the functional axis is selected and no authority is available, you ask once for acceptance criteria; you mark the axis not run only when none can be supplied.
8. **Map impact.** You identify changed public contracts, data paths, permissions, external inputs, tests, and callers that require surrounding context.
9. **Record baseline.** You separate pre-existing issues and unrelated user edits from the review target.

## Stop conditions

- You stop when the target cannot be resolved or validated.
- You stop before fetching unavailable private data, checking out a revision, or altering repository state.
- You return incomplete when truncation or inaccessible files prevent full coverage.
- You stop and ask for the base when repository metadata cannot distinguish one default branch.

## Test

- The collected scope names the target and files reviewed plus the exact base and head when the target has them.
- Every file belongs to a bounded batch and no file is silently omitted.
- A clean non-default branch still reviews its complete committed diff from the merge base.
- No repository or external state changes.
