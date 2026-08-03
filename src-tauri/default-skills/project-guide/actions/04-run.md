# 04 - Run

You carry out only the user's selected reply.

## Input

- Accept the user's reply, the rendered decision, the exact available capability map, and the session ledger.

## Output

- Return the requested read-only response, selected action result, safe handoff, or one-line close.

## Process

1. **Interpret reply.** You read [replies.md](../references/replies.md) and accept only a displayed key or supported word reply.
2. **Handle read-only replies.** You show details, explain one step, recap existing conversation context, or return to the prior screen without rescanning or mutating files.
3. **Guard selection.** You invoke only the exact capability displayed on the screen. You invoke nothing for a functional gap or instruction-only step.
4. **Run complete actions.** For `complete-now`, you load the selected capability, follow its complete contract, record the result, and rescan current evidence.
5. **Hand off interaction.** For `interactive-handoff`, you load the selected capability, state that the guide will resume from fresh evidence afterward, and let that workflow ask its own necessary questions.
6. **Walk pending steps.** For `OK`, you state the number of pending ranked steps, run only consecutive `complete-now` actions, and pause at the first interactive, instruction-only, unavailable, blocked, or failed step. You never walk an idle menu.
7. **Record ledger.** You record completed, explicitly skipped, reviewed, or instruction-only steps in session context. You never persist the ledger to project files.
8. **Rescan selectively.** You rescan after a mutating or state-changing action. You do not rescan after details, back, recap, explanation, category expansion, or stop.
9. **Return safely.** You propagate a failed selected action as blocked. You never mark it completed merely because control returned.

## Stop conditions

- You stop when the reply does not match a displayed option and ask the user to choose again.
- You stop before invoking an unavailable, ambiguous, destructive, or unselected capability.
- You stop the walk on the first failure, blocker, required interaction, or instruction-only step.

## Test

- Read-only replies change no file and do not trigger a rescan.
- A functional gap invokes nothing and names only the missing function.
- `OK` never walks idle choices and pauses at the first non-complete action.
- A failed action remains blocked in the next scan.
