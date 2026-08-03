# 03 - Integrate

You incorporate the latest answer and decide whether another question round is necessary.

## Input

- Accept the previous idea summary.
- Accept the user's latest answer.

## Output

- Return an updated intent-level definition.
- Return a decision to revisit `02-probe` or continue to `04-finalize`.

## Process

1. **Validate.** You read long answers in ordered chunks of at most 100,000 characters and process at most 50 newly stated items per batch. You carry confirmed context forward and continue until the complete answer is covered before you judge whether it resolves the active question fully, partly, or not at all.
2. **Update.** You incorporate confirmed information and preserve earlier decisions that the answer does not change.
3. **Expose.** You state any contradiction, new assumption, or new fork opened by the answer.
4. **Lean.** You state a preferred direction and its tradeoff only when the confirmed facts clearly favor it. You do not lock the implementation.
5. **Judge.** You ask whether two competent readers could still produce materially different intended outcomes from the current definition.
6. **Route.** You return to `02-probe` when the answer is yes. You continue to `04-finalize` when the answer is no or the user asks to conclude.

## Stop conditions

- You stop and clarify when the answer contradicts a confirmed requirement and the user's intended replacement is unclear.
- You do not count rounds or continue only to reach a fixed number of questions.

## Test

- The updated definition reflects the latest answer without losing prior confirmed decisions.
- The route decision depends on material ambiguity, not a round count.
- Every unresolved claim remains explicitly marked as open.
