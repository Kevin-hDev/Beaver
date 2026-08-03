# 02 - Write

You render the confirmed role without weakening or widening it.

## Input

- Accept the confirmed contract, selected name, target runtimes, paths, and write boundary from action 01.

## Output

- Return the canonical role, every file written or preserved, and the conversion applied per runtime.

## Process

1. **Build canonical form.** You copy [agent-template.md](../assets/agent-template.md), remove every placeholder, and express the confirmed role in short imperative instructions.
2. **Render native form.** You write `.beaver/agents/<name>.md` with only `name`, `description`, and `profile`. You keep `profile` equal to `explorer` or `coder`.
3. **Render external forms.** You apply only the confirmed conversions in [runtime-targets.md](../references/runtime-targets.md). You omit unsupported optional fields instead of inventing replacements.
4. **Preserve content.** You merge only where the target format requires a shared file. You preserve unrelated agents, configuration, comments, and user customizations.
5. **Validate paths.** You reject absolute destinations, `..`, symlink escapes, duplicate targets, and filenames that do not match the confirmed slug.
6. **Report writes.** You list each created, updated, unchanged, or skipped path without claiming runtime validity yet.

## Stop conditions

- Stop before writing when the canonical role still contains placeholders, ambiguous permissions, secrets, several responsibilities, or an unconfirmed model or tool grant.
- Stop when a target cannot represent a required capability without semantic loss; report that target as `blocked` instead of silently dropping the capability.

## Test

- Verify that each written file carries the confirmed name, description, and complete role body.
- Verify that every target preserves the same responsibility, output, guardrails, and effective capability boundary.
