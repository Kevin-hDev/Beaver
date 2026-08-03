# 06 - Synchronize

You deterministically refresh existing marked project context references after approved memory or ADR writes.

## Input

- Use the successful memory or ADR delivery summary, approved synchronization plan, and resolved project-local context conventions.

## Output

- Return the refreshed memory index, synchronized context files, enumerated references, validation results, independent review verdict, and proof that staged state was unchanged.

## Process

1. **Read the synchronization contract.** You read [synchronization.md](../references/synchronization.md) before touching a context file.
2. **Confirm applicability.** You skip synchronization for rule-only or skill-only handoffs and for memory or ADR no-ops.
3. **Snapshot staged state.** You record the current staged-path and staged-content identity with read-only operations when the project uses version control. You never run a staging command.
4. **Resolve the memory index.** You identify the existing project-local memory index and its established entry format. You ask when the index is missing or ambiguous and never invent one silently.
5. **Resolve marked blocks.** You use only existing project context files and their existing managed marker pairs. You ask before adding a missing marker or context file.
6. **Enumerate current references.** You canonicalize the resolved project-local memory and ADR roots, process at most 200 files per numbered batch, continue until complete, and sort normalized relative paths deterministically.
7. **Render replacements.** You refresh the memory index with the complete current memory file set in its established format. You replace only the content inside each approved context marker pair, preserve all outside bytes, and render every target to a sibling temporary file.
8. **Validate before replacement.** You verify every referenced file exists, every target still matches the version read, marker pairs are unique and balanced, unrelated bytes are unchanged, and all numbered batches are complete.
9. **Replace atomically.** You replace the index and each validated context file atomically, stage nothing, and stop on the first replacement failure with every untouched target preserved.
10. **Verify current files.** You re-read the memory index and every changed context file, compare them with the complete sorted source lists, and run the independent review contract.
11. **Verify staged state.** You compare the final staged identity with the initial snapshot and report failure if any difference appeared.

## Stop conditions

- You stop when the memory-index convention, context conventions, marker pairs, roots, or approved targets are missing, ambiguous, unsafe, or changed concurrently.
- You do not create a context file, marker, memory bank, or ADR taxonomy silently.
- You do not rewrite an entire context file, preserve stale references knowingly, or stage any path.

## Test

- The existing memory index contains the complete current deterministic memory-file set, and every approved marked block contains the complete current project-local reference set across all batches.
- Text outside approved markers remains byte-for-byte unchanged and every link resolves.
- The independent review verdict is `pass` and the staged state is identical before and after synchronization.
