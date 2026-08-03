# 03 - Present

You render one compact decision screen and wait for the user's reply.

## Input

- Accept the current decision, its evidence, state class, choices, and first-screen status.

## Output

- Return one rendered screen followed by the user's reply, with nothing appended after the options line.

## Process

1. **Load shape.** You use [screen-template.md](../assets/screen-template.md) and choose exactly one state shape.
2. **Frame once.** You add one short orientation sentence on the first screen of the guide run and omit it on later screens.
3. **Show state.** You summarize every applicable foundation as `met`, `drift`, `missing`, `blocked`, or `unknown`. You never present an uninspected surface as absent.
4. **Show one action.** You render exactly one primary action block with one key, its reason, and its execution behavior. You keep secondary choices on one compact options line.
5. **Explain drift.** You give every drift or blocker one evidence-backed cause and one keyed action or functional gap.
6. **Hide detail.** You keep exact capability identifiers, secondary evidence, lookahead, and batch notes behind the details reply.
7. **Use safe keys.** You use unique letters or digits and repeat no key on the screen.
8. **Wait.** You finish with the options line, print nothing afterward, and wait for an explicit reply.

## Stop conditions

- You stop before presenting when the decision has no evidence or contains an invented action.
- You do not render several primary action blocks, a project dump, or an unsolicited execution log.

## Test

- The first screen alone contains the orientation sentence.
- The screen contains exactly one primary action and each key appears once.
- Every drift or blocker includes its cause.
- Nothing appears after the options line.
