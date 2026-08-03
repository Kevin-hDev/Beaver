# 03 - Drill

You descend one selected axis or item exactly one level at a time.

## Input

- Accept one selected Tooling, Context, or Codebase scope.
- Accept an optional goal and the current depth when the action continues.

## Output

- Return a complete bounded listing of the current level, an optional single best match for a stated goal, and the available next levels.

## Process

1. **Detect cold entry.** Detect available surfaces when the survey did not run first.
2. **Set one level.** Treat an axis as its present surfaces, a surface as its items, and an item as its immediate contents or relationships.
3. **List completely.** Enumerate the selected level in ordered batches of at most 20 areas, 50 files, or 100 matches. Record the last stable item or cursor after each batch and continue at the same level until the requested enumeration is complete or a true blocker prevents progress. For Tooling, include item, location or catalog identity, purpose, and invocation path when one exists. For Context, include source, scope, and freshness evidence. For Codebase, include child modules, entry points, responsibilities, and representative tests.
4. **Match a stated goal.** When the user supplies a goal, compare current items and name one best match. Mention a second only when evidence produces a genuine tie.
5. **Point without running the target.** Return the exact invocation, catalog identity, file, symbol, or child scope. Use read-only search, symbol inspection, or graph queries as needed, but never execute the discovered target capability.
6. **Offer depth.** Name the available child levels, parent level, and other axes. Wait for the user's selection before descending again.

## Stop conditions

- Stop at a leaf, on request, after completing the requested level, or at a true evidence or access blocker.
- Start the next same-level batch when the current batch reaches 20 areas, 50 files, or 100 matches. Report the continuation position and never treat a batch limit as silent completion.
- Continue in a later pass when the user selects a child level; do not auto-expand every leaf.
- Stop and report ambiguity when equally named items would lead to different scopes.
- Do not turn the best match into generalized workflow advice.

## Test

- Confirm that the action lists exactly one level and only present items.
- Confirm that every same-level batch respects its ceiling, records its continuation position, and continues until the requested level is complete or explicitly blocked.
- Confirm that a stated goal yields at most one best match plus one genuine tie.
- Confirm that no discovered target capability was executed and the user controls every deeper descent.
