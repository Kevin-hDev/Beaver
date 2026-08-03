# 03 - Sketch UI

You sketch the structural layout needed to remove UI ambiguity before you split the work.

## Input

- Use the confirmed UI behavior and the existing interface structure found during exploration.

## Output

- Return one low-fidelity text sketch per affected screen or state with numbered region notes.

## Process

1. **Select.** You list only the screens or states the confirmed scope changes. You skip this action when the work has no user interface.
2. **Read.** You inspect [ui-sketches.md](../references/ui-sketches.md) and the existing component layout.
3. **Draw.** You place structural regions, controls, content, empty states, errors, and responsive changes that affect layout.
4. **Label.** You add one concise note for each numbered region and connect it to a confirmed behavior.
5. **Confirm.** You ask for approval when the sketch resolves a layout choice the source did not already settle.

## Stop conditions

- You stop before inventing final copy, colors, visual style, or behavior.
- You stop and ask when two layouts would materially change the user journey.
- You do not edit interface files.

## Test

- Every region maps to confirmed scope or inspected existing structure.
- The sketch communicates hierarchy without prescribing final styling.
- Non-UI work produces no invented sketch.
