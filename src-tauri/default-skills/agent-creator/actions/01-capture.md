# 01 - Capture

You settle the specialist contract and every destination before writing.

## Input

- Accept a free-form purpose, optional existing definition, requested tools, outputs, constraints, and target runtimes.

## Output

- Return a confirmed role, three candidate names, selected name, inputs, output, guardrails, runtime targets, native profile when applicable, and exact write paths.

## Process

1. **Inspect.** You read the existing definition and neighboring agent conventions when refactoring or when the project already contains agent directories.
2. **Clarify.** You identify the single responsibility, invocation situations, inputs, output, forbidden decisions, required skills, and runtime targets.
3. **Bound capabilities.** You select `explorer` for read-only work or `coder` for isolated changes in the native runtime. You include only tools and skills the role actually needs elsewhere.
4. **Draft.** You outline the canonical role using [agent-authoring.md](../references/agent-authoring.md).
5. **Name.** You propose three short names, explain collisions, and let the user select one.
6. **Resolve targets.** You inspect [runtime-targets.md](../references/runtime-targets.md), detect only evidence that exists, and ask the user to confirm each runtime and exact project-relative path.
7. **Confirm writes.** You state every create, merge, replace, or preserve operation and wait for confirmation when any existing file would change.

## Stop conditions

- Stop when the role contains several independent responsibilities, a requested tool exceeds the role, a target runtime is unsupported, the destination escapes the project, or an overwrite remains unconfirmed.
- Stop without writing when the request is only a one-off delegation; return a bounded delegation prompt instead.

## Test

- Verify that the confirmed contract names one responsibility, one output, explicit guardrails, every target, and every path.
- Verify that the selected native profile matches the requested effects.
