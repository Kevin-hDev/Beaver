---
tracking_version: 1
task: "{task slug}"
objective: "{observable end state}"
status: pending
completion: unverified
blocker_kind: none
attempt: 0
max_attempts: "{finite total}"
batch_size: "{finite attempts per batch}"
deadline_utc: "{deadline or bounded duration start reference}"
no_progress_limit: "{finite consecutive threshold}"
consecutive_no_progress: 0
state_fingerprint: "{branch, revision, and relevant state identity}"
---

# Persistent run: {title}

## Project evidence

- Environment and versions: `{relevant runtime, tools, and versions}`
- Applicable instructions: `{project-local rule or instruction paths}`
- Expected reads: `{artifacts}`
- Expected creations: `{artifacts or none}`
- Expected edits: `{artifacts or none}`
- Expected removals explicitly requested: `{artifacts or none}`

## Success predicate

- Kind: `{command|observable}`
- Working directory or target: `{validated target}`
- Procedure: `{executable plus separate arguments, or deterministic observation steps}`
- Pass result: `{exact expected exit/result}`
- False-positive guards: `{checks that prove the intended work actually ran}`

## Action contract

- Scope: `{exact paths, systems, data, environments}`
- Allowed operations: `{explicit operation classes}`
- Forbidden effects: `{destructive, financial, credential, publication, and unrelated effects}`
- External effects included by original request: `{exact effects or none}`
- Gated-effect limits: `{exact target, maximum and remaining occurrences, safeguards, cost ceiling if relevant, recovery, verification, or none}`
- Time boundary: `{deadline or wall-time budget}`
- Resource boundary: `{bounded processes, output, quota, and cost}`
- Progress signals: `{measurable signals}`
- Preservation plan: `{existing work and recovery method}`

## Prerequisites

| Item | State | Evidence or blocker |
| --- | --- | --- |
| `{tool, data, access, or secret}` | `{available|obtainable within contract|blocked}` | `{redacted evidence}` |

## Journey map

| Step | Dependency | Acceptance check | Status |
| --- | --- | --- | --- |
| `{step}` | `{dependency or none}` | `{direct check}` | `[ ]` |

### Decision diagram

<!-- Include a compact diagram only when the journey contains a meaningful branch. Otherwise write `none`. -->

## Risks and alternatives

| Risk or assumption | Evidence to watch | Alternative hypothesis |
| --- | --- | --- |
| `{risk}` | `{signal}` | `{different approach}` |

## Contract amendments

<!-- Append user-authorized scope or boundary changes. Never rewrite the original contract. -->

## Attempt log

<!-- Append one entry from references/attempt-log-format.md per attempt. Never rewrite history. -->

## Completion evidence

<!-- Fill only after the exact success predicate passes in the current state. -->

## End-to-end validation demonstration

<!-- Record the shortest reproducible path a real user or independent executor can follow to validate the completed outcome. -->
