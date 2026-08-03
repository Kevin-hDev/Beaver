# 01 - Discover

You define the smallest complete validation matrix for the implementation.

## Input

- Accept a changed working tree, commit, diff range, plan phase, file set, described implementation, or an explicit whole-project architecture conformance request.

## Output

- Return the resolved scope and a matrix of required, optional, and not-applicable code, architecture, and UI gates.

## Process

1. **Validate.** You validate the repository root, revision syntax, file paths, and supplied scope before inspection. You reject traversal and ambiguous revisions.
2. **Resolve scope.** For changed work, you inspect at most 50 changed files and 500 KiB of diff text per batch and continue additional batches until the requested scope is covered. For explicit whole-project architecture conformance, you resolve the repository as the scope and mark the architecture facet `global-report-only`.
3. **Read rules.** You load the project instructions and configuration that apply to the changed files.
4. **Trace impact.** You map the changed behavior to its callers, tests, data boundaries, build targets, architecture constraints, and visible UI paths.
5. **Classify gates.** You use [gate-selection.md](../references/gate-selection.md) to mark at most 50 gates in one batch as required, optional, or not applicable with one reason. You continue later matrix batches until every applicable gate is classified and executed.
6. **Choose commands.** You select repository-defined commands and pass their arguments separately without a shell intermediary.
7. **Record baseline.** You note known pre-existing failures only when evidence proves they predate the requested work.

## Stop conditions

- You stop when the scope, repository, or applicable rules cannot be resolved.
- You stop before inventing a command, downloading a tool, installing a dependency, or changing project configuration.
- You do not run code or modify files during discovery.

## Test

- Every selected gate traces to changed scope or an applicable project rule.
- Every required gate names an executable project command or an observable manual check.
- Unrelated working-tree changes remain outside the validation scope.
- A global architecture request is explicitly marked report-only and is not narrowed to changed files.
