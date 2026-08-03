# 01 - Orient

You establish the smallest safe scope and the available evidence sources.

## Input

- Accept a project root or the current workspace.
- Accept optional axes, items, behaviors, relationships, architecture questions, or capability goals.

## Output

- Return the validated root, requested axes, ordered exploration actions, depth, and bounded source plan.

## Process

1. **Validate root.** Canonicalize the project root, reject traversal, and confirm that it is a directory inside the allowed workspace.
2. **Read instructions.** Locate root instructions and only the nested instructions that govern likely sources.
3. **Classify scope.** Select `survey` for an open overview, `drill` for each named axis or item, and `trace` for each named behavior, symbol, path, or relationship. Select every applicable action instead of forcing one exclusive route.
4. **Order combined work.** Run Survey before Drill and Trace when the overview supplies their context. Preserve the user's requested order when the actions are otherwise independent.
5. **Detect surfaces.** Inspect session catalogs and project signals from [tooling-surfaces.md](../references/tooling-surfaces.md). Include only available Tooling, Context, and Codebase surfaces.
6. **Find project signals.** Inspect only manifests, workspace definitions, primary configuration, and documentation indexes needed to recognize the stack and layout.
7. **Set bounds.** Cap the pass at 20 top-level areas, 50 relevant files, 100 search matches, and one project root.
8. **Select sources.** Prefer project instructions, active catalogs, manifests, public entry points, symbols, and tests over generated summaries or assumptions.

## Stop conditions

- Stop when the root is missing, outside the allowed workspace, or ambiguous between multiple roots.
- Stop before reading a secret, credential store, environment file, database dump, or unrelated personal file.
- Narrow the level when the requested exploration cannot fit within the bounds.
- Ask which axis to inspect only when the request is open and a three-axis survey would not answer it safely.

## Test

- Confirm that the output identifies one root, one or more ordered exploration actions, available axes, and sources for the next action.
- Confirm that no project file changed and no sensitive file opened.
