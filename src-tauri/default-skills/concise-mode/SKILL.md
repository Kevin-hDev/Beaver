---
name: concise-mode
description: Sets, inspects, changes, or disables a session-local concise response level and reports honest usage statistics. Use for lite, full, or ultra brevity across replies. Not for shortening artifacts, code, quotes, or one-off prose edits.
---

# Concise Mode

You control the default concision of assistant prose for the current conversation without reducing correctness, evidence, or requested depth.

## Actions

You read every action used by the selected path before you run it.

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-configure`](actions/01-configure.md) | You enable, change, toggle, or disable the mode | A stable state confirmation |
| [`02-inspect`](actions/02-inspect.md) | You inspect the current level or whether the mode is active | The resolved session state and its evidence |
| [`03-stats`](actions/03-stats.md) | You inspect observed mode usage or response-volume statistics | A measured, estimated, or unavailable statistics report |

## Rules

- You keep the mode session-local. You resolve its state only from the current conversation and never claim persistence across conversations.
- You treat the latest valid assistant-authored state confirmation as authoritative: `Concise mode: ON (lite|full|ultra).` or `Concise mode: OFF.`
- You resolve missing or conflicting state as off and state that no active setting was found.
- You apply an active level to subsequent assistant prose until the user changes or disables it. You use `full` when the user enables or toggles on without naming a level.
- You apply the level as a presentation preference, not a content limit. You preserve required evidence, rationale needed to support a conclusion, citations, safety warnings, blockers, assumptions, code, quoted text, validation results, and every detail the user explicitly requests.
- You expand any passage whose compression could cause ambiguity, unsafe action, incomplete instructions, or user confusion. You resume the active level after that passage.
- You leave code, commands, error text, quotations, commit messages, pull-request text, and other requested artifacts unchanged unless the user separately asks to edit that artifact.
- You obey an explicit request for a detailed tutorial, full explanation, or specified format while the mode remains active for later replies.
- You distinguish measured values from estimates. You never infer savings from generic compression percentages or compare unlike replies as though they shared a baseline.
- You report savings only when a real comparable baseline and a documented counting method are available. Otherwise, you return `Savings: unavailable (no comparable baseline).`

## Resources

- You read [levels-and-guards.md](references/levels-and-guards.md) before applying or changing a level.
- You read [measurement.md](references/measurement.md) before reporting statistics.
