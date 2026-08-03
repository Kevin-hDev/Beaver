# 01 - Detect Context

You identify the target repository and prove which pipeline capabilities are genuinely available.

## Input

- Accept the intended project path or use the current project when it is unambiguous.
- Use the current repository, installed tools, project instructions, and existing pipeline files.

## Output

- Return the canonical repository root, current and default branches, default-branch revision, remote identity, and working-tree state.
- Return each tracker, version-control, change-request, integration, scheduler, and development-workflow adapter with `available`, `unsupported`, or `unknown` capability evidence.
- Return existing configuration and generated-artifact paths without exposing credential values.

## Process

1. You canonicalize the target path, reject traversal and ambiguous repositories, and confirm the resolved root contains the expected version-control metadata.
2. You inspect project instructions before you infer file locations, commands, or naming conventions.
3. You identify the current branch, working-tree changes, remotes, and default branch through read-only version-control queries. You record the observed default revision.
4. You inspect installed tool schemas, executable help, project-owned adapter configuration, and current official documentation when an interface may have changed.
5. You discover tracker and change-request capabilities for item reads, paginated discussions, dependency reads, conditional state changes, comments, reactions, thread replies, and thread resolution.
6. You discover integration capabilities for event filters, concurrency, secure references, result artifacts, and deterministic finalization.
7. You discover scheduling capabilities for create, inspect, disable, cadence, and overlap control.
8. You discover a complete development-workflow capability from its advertised plan, implementation, test, review, commit, and change-request contract. You do not match a hardcoded name.
9. You inspect existing async-development configuration and generated artifacts without modifying them.
10. You return `unsupported` for every required operation whose real format or capability you cannot verify.

## Stop conditions

- You stop when the repository root, default branch, or remote target is ambiguous.
- You stop when path validation fails or a required read-only query cannot be trusted.
- You stop before setup effects when no usable tracker or version-control adapter is available.

## Test

- You confirm every reported path resolves inside the intended repository.
- You confirm the default revision through a second read-only query.
- You confirm an unavailable adapter is reported as `unsupported` with evidence instead of receiving a guessed interface.
- You confirm the report contains credential reference names at most and never a secret value.
