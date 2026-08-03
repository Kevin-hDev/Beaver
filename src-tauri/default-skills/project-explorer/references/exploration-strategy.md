# Exploration Strategy

Read this reference while you select sources, exclusions, or depth.

## Evidence order

1. Read applicable project instructions and active session catalogs.
2. Inspect root manifests, workspace definitions, build configuration, and public entry points.
3. Search capability names, symbols, routes, events, data types, and direct references.
4. Read the smallest relevant sections and representative tests.
5. Consult project documentation for intent, then verify current claims against code or configuration.
6. Use a project knowledge graph for cross-module relations only when it exists and direct search would lose the connection.

## Default exclusions

- Exclude dependencies, build output, caches, coverage, generated bindings, vendored code, lockfile contents, binaries, archives, and large datasets.
- Include an excluded area only when the request explicitly targets it.
- Never open environment files, credential files, key stores, browser profiles, database dumps, or secret-bearing runtime configuration.

## Invocation boundary

- Invoke read-only exploration tools such as file search, symbol inspection, reference lookup, and available project knowledge-graph queries.
- Do not execute, install, authenticate, configure, or modify the discovered target capability.
- Distinguish inspecting a declared invocation path from running that path.

## Depth selection

| Request | Useful depth | Evidence |
| --- | --- | --- |
| What capabilities are available? | Tooling surface and items | Session catalogs, project instructions, present configuration |
| What context is retained? | Context sources and immediate contents | Instructions, memory, specs, plans, indexes |
| What is this project? | Codebase roots and major areas | README, manifests, entry points |
| Expand this skill or module | One drill level | Catalog metadata, child files, exports, representative tests |
| Where is this behavior? | Direct symbol and references | Definitions, callers, tests |
| How does data reach storage? | One ordered path | Entry point, validation, service, storage boundary |

## Evidence hygiene

- Cite stable contracts before incidental references.
- Quote only the minimum needed to disambiguate a claim.
- Do not infer runtime reachability from an import alone.
- Do not infer correctness, security, or maintainability from structure alone.
- Treat catalog absence and filesystem absence separately.
