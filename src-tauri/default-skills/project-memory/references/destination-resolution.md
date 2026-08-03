# Destination Resolution

Resolve project-local memory and instruction targets deterministically.

## Existing convention

1. Inspect repository-owned root instruction files and project documentation for an explicit memory-bank path and instruction-file list.
2. Inspect existing project-local directories for a structured bank containing `README.md` and at least two recognized core destination names from [memory-map.md](memory-map.md).
3. Accept a convention only when one unambiguous memory root and one or more project instruction files are documented or already wired.
4. Ask the user to choose when multiple conventions conflict.

## No convention

Ask one question that requests both:

- the memory-bank destination relative to the project root;
- the project instruction files that should carry the synchronized section.

Do not invent a directory or instruction filename. Allow a new instruction file only when the user names it.

## Path validation

- Canonicalize the project root and every existing target.
- Canonicalize the nearest existing parent for a new target, then join only validated relative segments.
- Reject absolute user-supplied destinations, `..`, null bytes, control characters, symlink escapes, and paths outside the project.
- Never search home-level, personal, global, sibling-project, or tool-account memory locations.
