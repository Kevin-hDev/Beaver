# Inspect project health

Produce a read-only, evidence-backed snapshot of repository state and its declared validation checks.

## Why

**Read-only inspection** reveals local drift and failing checks before work begins without changing dependencies, files, or external systems.

## Steps to inspect project health

#### 1) 🧭 Inspect repository state

Capture the active work and untracked files so later results are not mistaken for changes made by the inspection.

1. Run the status command from the canonical project root.
2. Preserve the output as baseline evidence without staging or cleaning anything.

```bash
$ git status --short
 M src/example.ts
?? notes.txt
```

#### 2) 🔎 Discover declared checks

Read the project's own instructions and automation before choosing commands so the inspection uses the maintained contract.

1. Inspect project instruction files, build manifests, and continuous-integration configuration.
2. List the exact lint, type, test, and build checks that apply without installing or upgrading dependencies.

```text
Evidence: package scripts + continuous-integration workflow
Checks: lint, typecheck, unit tests
```

#### 3) ✅ Run safe available checks

Execute only checks that are already available and non-destructive, recording actual exit status instead of summarizing from memory.

1. Run each discovered check with direct arguments and no shell interpolation of untrusted input.
2. Record pass, fail, or unavailable plus the minimal failure evidence.

```text
lint: passed
typecheck: failed — 2 diagnostics
unit tests: unavailable — dependencies not installed
```

## Verify

- Confirm that the final snapshot includes the original repository status, evidence for selected commands, and an actual result for every attempted check.
- Confirm that no dependency, file, branch, commit, remote, or external service changed.
