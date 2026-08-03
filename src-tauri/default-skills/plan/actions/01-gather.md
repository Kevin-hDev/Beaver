# 01 - Gather

You resolve the planning source and restate its confirmed scope without proposing a solution.

## Input

- Accept a user request, specification, PRD, ticket, issue, or readable local file.

## Output

- Return the source type and reference.
- Return the objective, required behavior, hard constraints, non-goals, and unresolved blockers.

## Process

1. **Validate.** You validate the source type, path, and identifier before reading. You reject traversal, unreadable files, and unsupported references. You read at most 256 KiB from a file or 100,000 inline characters per source batch and continue later batches until the complete source is covered.
2. **Resolve.** You read the referenced source or use the inline request exactly as supplied. You do not invent missing content.
3. **Restate.** You process at most 50 source requirements per batch and preserve the remaining order for the next batch. You summarize the objective, required behavior, constraints, non-goals, and stated completion conditions. You replace every secret or sensitive value with its purpose and a redaction marker.
4. **Separate.** You distinguish confirmed facts from assumptions and unresolved decisions.
5. **Gate.** You ask one focused question and wait when an unresolved decision would materially change the plan. You continue with a labeled assumption only when the user explicitly permits it.

## Stop conditions

- You stop when no usable source is available.
- You stop when the source cannot be read or validated.
- You stop before choosing a product or architecture decision the source leaves open.

## Test

- The summary traces every hard constraint to the source.
- The summary contains no implementation phase, library choice, or invented requirement.
- No file or external system changes.
