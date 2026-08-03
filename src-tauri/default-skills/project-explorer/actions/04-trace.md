# 04 - Trace

You follow one named behavior, symbol, request, event, or data object across actual project relationships.

## Input

- Accept one validated concept or question and the applicable project instructions.

## Output

- Return an ordered path of entry points, transformations, boundaries, effects, and tests with evidence.

## Process

1. **Anchor the concept.** Identify the exact route, symbol, event, field, command, component, or test named by the request.
2. **Search references.** Collect at most 100 direct references and reduce them to at most 50 relevant files.
3. **Use exploration tools.** Invoke read-only search, symbol inspection, and an available project knowledge graph when they help prove the path. Never execute the target behavior.
4. **Follow direction.** Trace callers to callees, input to output, producer to consumer, or UI action to backend effect.
5. **Cross actual boundaries.** Include serialization, validation, storage, process, network, and state transitions only when the path crosses them.
6. **Confirm with tests.** Use existing tests as behavioral evidence and distinguish tested behavior from implementation facts.
7. **Close the path.** Stop at the requested outcome, external boundary, or evidence gap.
8. **Label inference.** Mark every relationship not directly proven by a file, symbol, catalog, or tool result.

## Stop conditions

- Stop and report ambiguity when several anchors match and choosing one changes the trace.
- Stop at unresolved dynamic dispatch, generated code, unavailable systems, or missing configuration.
- Never execute the target behavior while exploring it.

## Test

- Confirm that every edge cites source and destination evidence or carries an inference label.
- Confirm that the trace answers one path without unrelated branches or code changes.
