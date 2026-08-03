# Gap Identity

You assign a stable key to every gap so repeated analysis tracks the missing decision rather than its wording.

## Key fields

1. Use the gap category from the shared rubric.
2. Use the most stable available source anchor: requirement identifier, section heading, named field, or a normalized semantic anchor for inline text.
3. Use a concise normalized phrase naming the missing decision, actor, state, failure, boundary, data rule, dependency, or verification rule.

Represent the key as `category | source-anchor | missing-decision`. Normalize whitespace and case consistently. Preserve meaningful identifiers, numbers, and domain terms.

## Stability rules

- Keep the key unchanged when severity, consequence wording, evidence text, or direct-question wording changes.
- Change the key when the underlying missing decision, source anchor, or category changes materially.
- Normalize a renamed heading, actor label, or sentence to the same semantic capability anchor when it governs the same underlying decision across artifact versions.
- Assign a separate `ambiguities | <semantic anchor> | removal intent` key when an explicit earlier requirement disappears and no source confirms whether that removal is deliberate.
- Do not reuse an earlier requirement as an earlier gap. Scan each artifact independently before comparing its gap identities.
- Use quoted evidence as a legacy fallback only when the prior report contains no stable anchor or missing-decision field.
- Mark a match ambiguous when two gaps resolve to the same fallback identity. Do not guess.
