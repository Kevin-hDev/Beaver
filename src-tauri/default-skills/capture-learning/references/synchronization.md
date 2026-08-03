# Deterministic Context Synchronization

Synchronize only existing marked reference blocks that the project convention assigns to memory or ADR references.

## Reference set

1. Resolve project-local memory and ADR roots from current files.
2. Canonicalize each root and reject escapes or unsafe links.
3. Enumerate regular reference files in batches of at most 200 until complete.
4. Normalize references to project-relative paths using the existing link syntax.
5. Sort paths lexicographically and remove exact duplicates.

## Memory index

1. Resolve the existing project-local memory index and its established entry boundaries and format.
2. Render the complete current memory-file set across all batches, excluding the index itself unless the project convention includes it.
3. Preserve unrelated index content exactly and refuse to invent a missing index or taxonomy.
4. Verify every rendered index entry resolves and a second render is identical.

## Marked replacement

1. Require one unique balanced start and end marker pair per managed block.
2. Preserve the exact prefix before the start marker and suffix after the end marker.
3. Render only the deterministic reference list between markers.
4. Preserve newline style and unrelated text byte-for-byte.
5. Render all sibling temporary files and validate every target before any replacement.
6. Re-read each original immediately before its atomic replacement and stop on concurrent change.

## Verification

- Confirm every memory-index and context reference resolves to a current file and no current in-scope file is absent.
- Confirm ordering and formatting are deterministic on a second render.
- Confirm files outside approved marked blocks are unchanged.
- Confirm no staging operation ran and staged path/content identity is unchanged.
- Return the exact updated files, reference counts, skipped targets, limits, and independent review verdict.
