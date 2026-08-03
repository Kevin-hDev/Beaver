# State Model

Use current project evidence to classify each applicable check as `met`, `drift`, `missing`, `blocked`, or `unknown`.

## Foundations

| Check | Apply when | Met evidence | Drift or missing evidence |
| --- | --- | --- | --- |
| Technical vision | The project is greenfield or the user asks for architectural setup | An approved technical-vision or installation document exists, or established code proves the stack | No approved choice exists for a greenfield project |
| Durable project context | Any project contains code or an approved vision | A project-local memory bank contains current, non-placeholder content | The bank is absent, empty, placeholder-only, or contradicted by current code |
| Context references | Project instruction files are present or selected by the user | Their marked memory sections reference every current memory entry | The section is absent, incomplete, duplicated, or malformed |

Treat established source code as proof that a stack exists. Do not force a greenfield architecture workflow onto an existing codebase merely because a technical-vision file is absent.

## Delivery stages

Use cumulative evidence and select the furthest stage actually proven for the current work:

1. **Clarify:** the request remains materially ambiguous.
2. **Specify:** an accepted product or functional contract exists when one is needed.
3. **Plan:** an approved implementation plan exists.
4. **Implement:** code changes against the plan exist.
5. **Validate:** required checks have current passing evidence.
6. **Review:** the full requested change has a current verdict.
7. **Commit:** the accepted change is recorded when delivery includes version control.
8. **Review request:** the current branch has an open request when delivery includes one.

Ignore review requests and delivery state from unrelated branches.

## Plan-status hedge

| Status | Meaning |
| --- | --- |
| `pending` | Recommend implementation |
| `in-progress` | Keep implementation active |
| `implemented` | Recommend validation and review |
| `validated` | Recommend independent review |
| `reviewed` | Recommend the requested delivery step |
| `blocked` | Surface the blocker |
| Missing, unreadable, or contradictory | Mark the plan uncertain and request repair |

Use the project's documented status vocabulary when it differs. Never guess a mapping that could skip a gate.

## Health signals

| Signal | Evidence |
| --- | --- |
| Missing tests | No real test exists for source behavior in scope |
| Reported defect | A user report, failing check, or source `TODO`/`FIXME` tied to a real defect exists |
| Structural risk | Direct evidence shows a file, dependency path, or responsibility is materially more complex than comparable neighbors |

Keep a health signal unknown when generated files, fixtures, templates, or incomplete search results are the only evidence.

## Session ledger

Keep the ledger only in the current conversation. Mark a step handled when current evidence proves it done or the user completes, reviews, skips, or explicitly leaves it. Refresh mutable evidence after state-changing actions so the ledger never overrides current facts.
