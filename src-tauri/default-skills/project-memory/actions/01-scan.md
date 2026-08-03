# 01 - Scan

Inspect the project, resolve local destinations, and confirm evidence-backed capabilities without changing files.

## Input

- Accept a canonical project root and a setup or refresh request.
- Accept optional user-specified memory and project-instruction paths.

## Output

- Return the confirmed capabilities, exact memory destinations, chosen instruction files, evidence map, and scan limits.

## Process

1. **Validate the root.** Canonicalize the project root, reject traversal and symlink escapes, and snapshot unstaged, staged, and untracked state without changing it.
2. **Ground the project.** Find source code, manifests, tests, or project-owned documentation that explains the project. Stop when none exists.
3. **Resolve destinations.** Follow [destination-resolution.md](../references/destination-resolution.md). Use one unambiguous repository convention when it exists. Otherwise ask for both the memory-bank destination and the project instruction files, then wait.
4. **Read instructions.** Read only project-owned instruction files governing the root and resolved destinations. Stop if a required file is unreadable.
5. **Detect capabilities.** Apply [capability-signals.md](../references/capability-signals.md). Process at most 100 discovered paths and 50 evidence records per numbered batch, preserve the remaining queue, and continue until scanning is complete.
6. **Show evidence.** List every detected capability with concrete paths, manifest keys, or dependencies. Include `core` only after the project is grounded.
7. **Confirm the set.** Ask the user to confirm, remove, or propose capabilities. Accept an added capability only after locating repository evidence for it. Wait for confirmation before generation.
8. **Expand destinations.** Use [memory-map.md](../references/memory-map.md) to list every exact file selected under the resolved memory root.

## Stop conditions

- Stop without writing when the project has no code, manifest, test, or project description.
- Stop when the root, memory destination, instruction target, or required source escapes the project or cannot be read safely.
- Stop before generation until the capability set and ambiguous destinations are confirmed.

## Test

- Confirm that repository state matches the initial snapshot exactly.
- Confirm that every selected non-core capability has concrete repository evidence.
- Confirm that every destination is exact, project-local, unique, and mapped from a selected capability.
