# 02 - Generate or Refresh

Create a complete evidence-based memory bank or refresh an existing bank while preserving user work.

## Input

- Use the confirmed capability set, evidence map, resolved memory root, operation mode, and exact destination map from `01-scan`.

## Output

- Return a complete generated or refreshed memory bank, a preservation ledger, duplicate reconciliation, and obsolete-file candidates.

## Process

1. **Load contracts.** Read [memory-map.md](../references/memory-map.md), [memory-rules.md](../references/memory-rules.md), and [refresh-preservation.md](../references/refresh-preservation.md).
2. **Validate the bank.** Canonicalize the existing bank or its nearest existing parent, reject traversal and symlink escapes, and ensure the resolved destination remains inside the project root.
3. **Inventory safely.** Enumerate existing bank files in lexicographic order, at most 20 files per numbered batch, and continue until every file is classified. Record hashes and exact original text for preservation checks without reading outside the bank.
4. **Prepare the index.** Create a missing bank root and `README.md` from [memory-index-template.md](../assets/memory-index-template.md). Preserve every unrelated byte when the index already exists.
5. **Generate selected files.** Process at most 20 mapped files per numbered batch and continue until all selected rows are complete. Fill the exact template and destination from the map with repository evidence. Remove guidance comments, empty sections, placeholders, secret values, and unsupported claims.
6. **Refresh existing files.** Apply the preservation rules. Keep user text byte-for-byte, add supported facts only inside existing relevant sections, and flag stale or conflicting user text for approval instead of silently replacing it. Never restore a section the user removed.
7. **Reconcile duplicates.** Keep each fact in its canonical mapped home. Remove a duplicate automatically only when the current run introduced it. For pre-existing text, preserve or move its exact bytes first and require approval before removing the old copy.
8. **Flag obsolete files.** Flag a known mapped file when its capability is no longer selected. Leave unknown files untouched. Offer the exact obsolete paths and delete only the files the user explicitly names after revalidation.
9. **Refresh the index.** Enumerate every current Markdown memory file after approved pruning, sort project-relative paths, and replace only the `project-memory-files` marked section with one reference per file. Preserve all unrelated index text and reject malformed or duplicate markers.
10. **Write atomically.** Render and validate every candidate first, stage temporary files beside their destinations, replace each destination atomically, remove temporary files on failure, and leave the Git index unchanged.
11. **Verify preservation.** Compare the preservation ledger with final files and account for every original user-owned line as unchanged, explicitly approved, or still flagged.

## Stop conditions

- Stop before replacement when evidence is insufficient, a template remains incomplete, a destination collides, or preservation cannot be proven.
- Stop and ask before replacing or deleting any pre-existing user text.
- Stop after flagging obsolete files unless the user explicitly requests deletion.

## Test

- Confirm that `README.md`, all six core files, and every selected capability file exist at their exact mapped destinations.
- Confirm that the index references every current Markdown memory file exactly once and no obsolete deleted path.
- Confirm that no placeholder, guidance comment, secret value, unsupported claim, or duplicate introduced by the run remains.
- Confirm that every pre-existing user line survives byte-for-byte unless its exact change received approval.
- Confirm that no unselected, unknown, staged, personal, global, or cross-project file changed.
