# Decision Ranking

Choose the first applicable rank.

1. Select the earliest missing, drifting, or blocked foundation.
2. Select the earliest unmet delivery stage for the current work.
3. Select a fired project-health signal.
4. Build the idle menu only when ranks 1–3 are clear.

## State classes

| Class | Use it when | Presentation |
| --- | --- | --- |
| `greenfield` | No established code or approved technical vision exists | Show the first foundation |
| `existing` | Code exists but durable project context is missing | Show project-context setup |
| `drift` | A foundation exists but contradicts or incompletely references current evidence | Show one cause and repair action |
| `midwork` | A delivery stage is active | Show the stage and next gate |
| `blocked` | A required source, gate, or selected action failed | Show the blocker before normal choices |
| `idle` | Foundations and active delivery work are clear | Show one primary idle choice and compact alternatives |

## Idle categories

- **Start work:** use an available end-to-end workflow, or clarification when the idea is vague.
- **Improve health:** include available audit, test, refactor, or debugging capabilities whose evidence applies.
- **Customize project automation:** include available generators for project-local rules, skills, agents, commands, or automations.
- **Explore:** include an available project-exploration capability.

Show only categories with at least one available member. Treat categories as choices, never as an ordered walk.
