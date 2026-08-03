# 04 - Synchronize Context

Synchronize the current memory file list into chosen project instruction files without changing unrelated content.

## Input

- Accept a canonical project root and optional resolved memory root and instruction files.
- Use reviewed memory output when this action follows `03-review`.

## Output

- Return synchronized instruction files, complete reference verification, preserved-text verification, and the final run report.

## Process

1. **Resolve paths.** Follow [destination-resolution.md](../references/destination-resolution.md). Reuse one unambiguous project convention. Otherwise ask for both the memory-bank destination and chosen project instruction files, then wait.
2. **Require memory.** Validate the bank inside the project root and stop without writing unless it contains at least one readable Markdown memory file.
3. **Build the inventory.** Enumerate current Markdown files under the bank in sorted numbered batches of at most 100 paths. Reject traversal, symlink escapes, paths longer than 240 characters, and files outside the bank. Continue until all files are included.
4. **Render the section.** Follow [sync-contract.md](../references/sync-contract.md). Build one marked section with at most 100 references per numbered subsection and continue subsections until every current memory file appears exactly once.
5. **Reconcile markers.** Insert the section when absent, replace its contents when one pair exists, or keep the first position and remove only additional marked sections when duplicates exist. Preserve every byte outside owned marked sections.
6. **Prepare atomically.** Validate every instruction target inside the project, render all candidates, write temporary siblings, verify marker balance and complete references, and replace each target atomically only after all candidates pass.
7. **Verify results.** Read every target back. Confirm one marker pair, every current memory file exactly once, no stale memory reference, unchanged unrelated-text hashes, and an unchanged staged-file snapshot.
8. **Report.** Fill [report-template.md](../assets/report-template.md) once with setup, refresh, review, pruning, and synchronization results that apply to this path.

## Stop conditions

- Stop without creating an instruction file when the bank is empty, unreadable, outside the project, or unresolved.
- Stop before replacement when a marker is malformed, unrelated text would change, any current file is missing, or a target cannot be written atomically.
- Stop rather than choosing an instruction file or memory location for the user when no project convention exists.

## Test

- Confirm that each chosen instruction file contains exactly one balanced marked section and references every current memory file exactly once.
- Confirm that no stale reference remains and all text outside owned sections is byte-for-byte unchanged.
- Confirm that unchosen files, personal or global files, and the Git index remain unchanged.
- Confirm that an already exact synchronization performs no write and reports no change.
