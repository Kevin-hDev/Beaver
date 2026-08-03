# 01 - Scope

You define a safe, bounded audit and identify which checks can produce reliable evidence.

## Input

- Accept a project root and optional path scope.
- Accept one named pillar or a request for a complete audit.

## Output

- Return the validated scope, selected pillars, applicable project rules, available read-only checks, and explicit exclusions.

## Process

1. **Validate root.** You canonicalize the root and user-supplied paths, reject traversal, and remain inside the allowed project.
2. **Read instructions.** You load root rules and only the nested rules that govern the selected scope.
3. **Select pillars.** You run a named pillar directly and all seven when the user explicitly requests a complete audit. When the request says only `audit`, `health check`, or another unscoped general term, you ask once whether to run all seven pillars or one named pillar before scanning.
4. **Protect work.** You inspect repository status and treat every existing change as user-owned.
5. **Discover checks.** You identify read-only analyzers, tests, coverage, profilers, manifests, lockfiles, architecture sources, and an existing interface target.
6. **Set bounds.** You process at most 20 findings per pillar batch and 50 findings per merged report batch. You preserve the remaining ordered scope and continue numbered batches until coverage is complete. You retry one unchanged failed tool check at most twice before recording it unavailable with evidence.
7. **Resolve delivery.** You keep the report in the conversation by default. When the user requests report artifacts, you validate one destination directory for the selected pillar reports and merged report before scanning.
8. **Record exclusions.** You name missing tools, unavailable runtimes, unsupported areas, and checks that would mutate state.

## Stop conditions

- You stop when the root or scope is invalid or when the requested audit would expose data outside the project.
- You skip a check when it would install software, change files, start infrastructure, or contact an external service with project data.
- You do not silently replace an unavailable check with a stronger unsupported claim.

## Test

- Every selected pillar has at least one evidence source or an explicit skip reason.
- A requested report destination stays inside the allowed project and contains no traversal.
- No file, dependency, external service, or running environment is changed.
