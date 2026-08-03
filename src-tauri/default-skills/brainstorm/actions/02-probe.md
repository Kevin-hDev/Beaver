# 02 - Probe

You ask a focused question round about the most important unresolved thread.

## Input

- Accept the current idea summary.
- Accept the answers already provided.
- Accept the idea's functional, technical, or mixed level.

## Output

- Return one primary question and at most two tightly related subquestions.
- Return a pause for the user's answer.

## Process

1. **Read.** You read [probing.md](../references/probing.md) before the first round and whenever the thread stalls.
2. **Select.** You choose the unresolved point that could still lead to materially different outcomes. You prioritize a fork, hidden assumption, boundary, failure mode, or unverifiable success condition.
3. **Match.** You keep the question at the user's current level. You do not ask for implementation detail that belongs to planning.
4. **Challenge.** You state the relevant fork or assumption plainly. You use one fitting tactic from the probing reference when it improves clarity.
5. **Ask.** You ask one primary question. You add no more than two subquestions, and only when the same answer naturally covers them.
6. **Wait.** You stop and wait for the user's response.

## Stop conditions

- You stop before asking a question that the conversation already answered.
- You stop and hand off to `04-finalize` when no material ambiguity remains or the user asks to conclude.

## Test

- The round follows one live thread and contains one primary question plus no more than two tightly related subquestions.
- The question can change the definition of the idea, not merely its implementation plan.
- The response ends by waiting for the user.
