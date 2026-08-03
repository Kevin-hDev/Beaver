# 02 - Survey

You produce one compact map of every available axis.

## Input

- Accept the oriented root, available surfaces, project instructions, and selected signals.

## Output

- Return Tooling, Context, and Codebase sections. List only present items, with a short purpose and evidence for each.

## Process

1. **Survey Tooling.** Enumerate available skills, instructions and rules, active tools, connectors, MCP capabilities, plugins, hooks, commands, and knowledge tools from session catalogs and present project surfaces.
2. **Survey Context.** Locate project instructions, memory, specifications, plans, indexed documentation, and context files. State whether each source is present and what it covers without dumping its contents.
3. **Survey Codebase.** Derive languages, frameworks, packages, entry points, tests, major modules, storage, processes, and external boundaries from manifests and representative files.
4. **Group responsibilities.** Group at most 20 items per axis and support every stated responsibility with a catalog item, file, or symbol.
5. **Record gaps.** Distinguish verified facts, reasonable inferences, unavailable surfaces, and areas excluded by bounds.
6. **Expose depth.** Name the items that can be expanded one level deeper without choosing one for the user.

## Stop conditions

- Stop expanding after 50 relevant files or 100 direct matches in the current pass.
- Skip an unavailable axis surface without treating it as an error.
- Do not enumerate every file when module-level evidence answers the survey.
- Do not recommend, execute, install, authenticate, or configure any listed target capability. Invoke read-only exploration tools when needed to collect evidence.

## Test

- Confirm that every available axis appears and every listed item is actually present.
- Confirm that the map contains no recommendation, target execution, or project modification.
