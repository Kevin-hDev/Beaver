# 01 - Configure

You enable, change, toggle, or disable concise mode for the current conversation.

## Input

- Accept the user's requested operation: enable, set a level, change a level, toggle, or disable.
- Use the latest valid state confirmation visible in the current conversation when the operation depends on existing state.

## Output

- Return exactly `Concise mode: ON (<level>).` when you enable or change the mode, replacing `<level>` with `lite`, `full`, or `ultra`.
- Return exactly `Concise mode: OFF.` when you disable the mode.

## Process

1. You read [levels-and-guards.md](../references/levels-and-guards.md).
2. You scan assistant messages in the current conversation from newest to oldest for the latest valid state confirmation.
3. You treat the state as off when no valid confirmation exists.
4. You resolve an explicit `lite`, `full`, or `ultra` request as enabled at that level, whether the prior state was on or off.
5. You resolve only an explicit mode-disable request, such as `concise mode off`, `disable concise mode`, or `normal mode`, as off.
6. You resolve a toggle as off when the latest state is on and as on at `full` when the latest state is off or missing.
7. You resolve a plain enable request as on at `full` unless the user names another level.
8. You emit only the stable confirmation line for a configuration-only request.
9. You apply the chosen level to later assistant prose while preserving every guard in the reference.

## Stop conditions

- You stop and ask one focused question when the requested level is not one of `lite`, `full`, or `ultra` and the intended level cannot be inferred safely.
- You stop after the state confirmation when the user requested no other work.
- You do not invent prior state that is absent from the current conversation.

## Test

- You verify that enabling without a level returns `Concise mode: ON (full).`.
- You verify that an explicit level always sets that level, an active toggle disables, and a missing-state toggle enables `full`.
- You verify that the next answer keeps required evidence and expands safety-critical passages regardless of level.
