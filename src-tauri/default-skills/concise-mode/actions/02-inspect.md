# 02 - Inspect

You report the concise-mode state established in the current conversation.

## Input

- Accept a request to inspect the active state, level, or default presentation preference.
- Use the visible current conversation as the only state source.

## Output

- Return `Concise mode: ON (<level>).` and identify the latest visible state confirmation when the mode is active.
- Return `Concise mode: OFF. No active setting found in this conversation.` when no valid state exists or the latest valid confirmation is off.

## Process

1. You scan visible assistant messages from newest to oldest for `Concise mode: ON (lite).`, `Concise mode: ON (full).`, `Concise mode: ON (ultra).`, or `Concise mode: OFF.`.
2. You select the first valid confirmation found and ignore older confirmations.
3. You report the resolved state without changing it.
4. You identify the evidence by its relative position, such as `latest state confirmation`, without claiming access to hidden storage or metadata.
5. You state that the mode affects presentation only when the user asks what the state means.

## Stop conditions

- You stop after reporting the state when the user requested no statistics or state change.
- You return the explicit off-and-missing result when no valid state confirmation is visible.
- You do not accept a quoted user message as a state confirmation or infer an active level from response style alone.

## Test

- You verify that the newest valid confirmation wins after several level changes.
- You verify that inspection never toggles or changes the state.
- You verify that missing state produces an honest off result rather than a guessed level.
