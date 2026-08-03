# 01 - Capture

You restate the idea faithfully and determine the level at which to explore it.

## Input

- Accept the user's idea as free-form text.

## Output

- Return a short bullet summary of the idea.
- Return a private functional, technical, or mixed classification.
- Return a handoff to `02-probe`.

## Process

1. **Validate.** You read long idea input in ordered chunks of at most 100,000 characters and keep a continuation cursor until you reach the end. You confirm that the complete input contains an idea to clarify. If it does not, you ask what idea the user wants to explore and wait.
2. **Restate.** You process at most 50 stated facts, preferences, constraints, and assumptions per batch, carry confirmed context forward, and continue until every supplied item is covered. You summarize the problem, intended outcome, known actors, and stated constraints in short bullets. You preserve the user's meaning and vocabulary without repeating secrets or sensitive values.
3. **Separate.** You distinguish facts, preferences, and open assumptions. You do not choose a solution.
4. **Classify.** You identify whether the user is reasoning at a functional, technical, or mixed level. You keep later questions at that level.
5. **Handoff.** You read `02-probe` and start the first focused question round.

## Stop conditions

- You stop and wait when the message contains no identifiable idea.
- You do not search files, browse the web, or write a file during capture.

## Test

- The summary preserves the stated intent without adding a solution.
- Facts, preferences, and assumptions are not presented as the same thing.
- No file or external system changes.
