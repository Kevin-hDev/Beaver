# Tooling and Context Surfaces

Read this reference when you survey or drill into Tooling or Context. Use only surfaces available in the current session or project.

## Discovery order

1. Inspect the session-provided skill catalog for installed skills and their canonical locations.
2. Inspect the active tool catalog for local tools, provider connectors, apps, MCP capabilities, browser or computer controls, and project knowledge tools.
3. Inspect project instructions such as `AGENTS.md` and only the nested instruction files that govern the selected scope.
4. Inspect project-local skill, rule, hook, command, prompt, plugin, and connector configuration directories only when they exist.
5. Inspect project memory, specifications, plans, documentation indexes, and knowledge-graph entry points only when they exist.

## Normalized Tooling fields

- Record the capability name and kind.
- Record its catalog identity or project-relative location.
- Record its declared purpose from current metadata.
- Record its invocation path only when the environment exposes one.
- Record availability separately from configuration or authentication state.

## Normalized Context fields

- Record the source name and project-relative location.
- Record the scope it governs or describes.
- Record freshness only when a date, version, or current code comparison proves it.
- Record whether the source is instructions, memory, specification, plan, documentation, or generated index.

## Safety

- Do not open connector tokens, environment files, credential stores, or secret-bearing configuration.
- Do not infer a capability from a shared parent directory alone.
- Do not treat a global catalog item as project-configured without project evidence.
- Do not scan global plugin directories, unrelated personal locations, or unavailable third-party CLIs to replace a session or project surface that is not exposed.
- Invoke read-only exploration tools when needed to inspect a discovered item.
- Do not execute, install, authenticate, configure, or modify the discovered target item.
