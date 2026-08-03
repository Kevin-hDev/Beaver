# 04 - Finalize

You consolidate the clarified idea, obtain approval, and offer appropriate next steps without starting them.

## Input

- Accept the clarified idea and confirmed decisions.
- Accept remaining assumptions, risks, and deferred questions.

## Output

- Return an approved idea statement.
- Return scope, success conditions, assumptions, and risks.
- Return the user-chosen persistence result or no file change.
- Return plain-language next activities.

## Process

1. **Consolidate.** You write one coherent intent-level definition. You include the problem, actors, desired outcome, boundaries, constraints, and observable success conditions that the conversation established.
2. **Flag.** You list every unresolved assumption, material risk, and consciously deferred decision. You do not fill gaps with guesses.
3. **Approve.** You show the result and ask the user to confirm or correct it. You wait for the answer.
4. **Offer destinations.** After approval, you offer the conversation, one validated Markdown file, and one configured tracker record as equal choices. You do not name or invoke another skill.
5. **Persist.** You keep the result in the conversation, write one Markdown file, or update one configured tracker record only after the user explicitly chooses that destination.
6. **Validate the destination.** Before writing, you reject an empty path, a path containing `..`, or a destination outside the authorized working area. You preserve existing content unless the user approved replacing it.
7. **Point forward.** You suggest specifying, planning, or building the idea as appropriate. You do not start the next activity automatically.

## Stop conditions

- You stop when the user does not approve the consolidated idea.
- You stop before writing when the destination is missing, invalid, or unauthorized.
- You stop after presenting next activities; you do not invoke another skill automatically.

## Test

- The final result separates confirmed decisions from open assumptions and risks.
- The user approves the result before any persistence.
- No file changes when the user chooses to keep the result in the conversation.
- A requested write targets only the validated destination.
