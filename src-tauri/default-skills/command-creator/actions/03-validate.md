# 03 - Validate

You independently check the command bundle's identity, compactness, triggering, input handling, and observable result.

## Input

- Use the confirmed command contract, final bundle, prior contents when available, and local validation tools.

## Output

- Return one status per intended file, a trigger-discrimination verdict, a one-objective verdict, and exact corrections or blockers.

## Process

1. **Read current files.** You rebuild the validation context from the final bundle and confirmed contract rather than trusting the write summary.
2. **Validate identity.** You check folder, frontmatter name, slash slug, evaluation identifier, description length, and optional UI prompt for exact agreement.
3. **Validate shape.** You read [command-authoring.md](../references/command-authoring.md), require one objective and at most eight direct steps, and reject unnecessary action files or supporting resources.
4. **Validate input.** You confirm the command consumes remaining user text, handles missing or invalid input honestly, and contains no foreign argument syntax.
5. **Validate behavior.** You evaluate positive, negative, tool-failure, no-change, and observable-result cases and distinguish keyword matches from semantic success.
6. **Validate preservation.** You compare refactored files with prior contents and flag unrelated deletion, movement, or expanded effects.
7. **Run checks.** You run available bundle validation and JSON parsing, capture failures, apply only confirmed mechanical corrections, and rerun affected checks.
8. **Close honestly.** You mark success only when every required check has evidence and otherwise return `failed`, `blocked`, or `skipped` with the reason.

## Stop conditions

- You stop a passing verdict when identity, path safety, compactness, input handling, trigger discrimination, or observable behavior is missing or unverifiable.
- You stop before a correction that changes the confirmed objective, side effects, destination, exclusions, or overwrite boundary.
- You do not treat a skill bundle as a built-in command or claim the application itself was changed.

## Test

- You verify that every intended file and required behavior has one evidence-backed status.
- You verify that all executed validators pass after corrections and unavailable checks remain explicit.
- You verify that the final command remains a single-objective slash-invokable skill rather than a hidden workflow or tool integration.
