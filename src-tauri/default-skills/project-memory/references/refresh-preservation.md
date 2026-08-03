# Refresh Preservation

Treat every pre-existing memory file as user-owned.

## Preserve

1. Record the original bytes, heading sequence, and line hashes before refresh.
2. Keep every existing non-placeholder line byte-for-byte unless the user approves its exact replacement or deletion.
3. Add newly supported facts only under an existing relevant heading.
4. Report a useful missing template section instead of recreating a heading the user removed.
5. Flag a stale or contradictory statement with decisive evidence and request approval before changing it.

## Reconcile duplicates

- Remove a duplicate automatically only when the current run created both copies.
- Keep the fact in the canonical home from [memory-map.md](memory-map.md).
- Preserve or move pre-existing text byte-for-byte before requesting removal of its old copy.
- Leave ambiguous duplicates in place and report both locations.

## Prune

- Treat only exact mapped destinations as known generated memory files.
- Flag a known file when its capability is no longer confirmed.
- Leave unknown files untouched.
- Delete only an exact, revalidated project-local path that the user explicitly names.
