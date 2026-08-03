# 04 - Validate

You independently validate the written bundle's structure, trigger boundary, workflow, resources, and evaluations.

## Input

- Use the confirmed frame and plan, final bundle files, prior contents when available, and local validation tools.

## Output

- Return one evidence-backed status per intended file, one trigger-discrimination verdict, one workflow verdict, and the exact corrections or blockers.

## Process

1. **Rebuild context.** You re-read the final files and confirmed contract instead of trusting the write summary.
2. **Validate structure.** You read [validation-protocol.md](../references/validation-protocol.md) and check frontmatter, naming, description length, required action sections, bounded resources, links, and placeholder absence.
3. **Validate behavior.** You trace every workflow branch and ensure each action can reach its output or an honest stop without consuming a later result.
4. **Validate triggering.** You compare the description with every positive and negative case and flag ambiguous overlap with neighboring skills.
5. **Validate preservation.** You compare refactored files with their prior contents and flag any unrelated deletion, weakening, or restructuring.
6. **Run checks.** You run the available bundle validator and evaluation parser. You capture exact command outcomes and do not suppress failures.
7. **Correct narrowly.** You fix only mechanical defects inside the confirmed boundary, then rerun every affected check. You return to the relevant earlier action for semantic changes.
8. **Close honestly.** You report `passed` only when every required file and behavioral check has evidence; otherwise you report `failed`, `blocked`, or `skipped` with the reason.

## Stop conditions

- You stop successful completion when a required file is missing, malformed, outside the destination, semantically weaker, untested, or unverifiable.
- You stop before a correction that changes meaning, scope, destination, action ownership, or the confirmed trigger boundary.
- You do not convert partial validation, an unavailable tool, or a keyword-only match into a pass.

## Test

- You verify that every intended file has exactly one evidence-backed status.
- You verify that positive and negative cases support a closed trigger-discrimination verdict.
- You verify that all executed checks pass after corrections and that unexecuted checks remain explicitly marked.
