---
workflow_id: <stable-local-id>
objective: <one sentence>
status: pending
plan_identity: <conversation-id-or-project-local-location>
delivery_contract:
  commit: false
  publish_branch: false
  change_request: false
---

# Workflow ledger

## Acceptance criteria

- [ ] <observable criterion>

## Sources

| Source | Resolved | Retained requirements | Notes |
| --- | --- | --- | --- |
| <safe reference> | yes/no | <identifiers> | <redacted note> |

## Plan

- Location or conversation identity: <value>
- Validation: <observed result>
- Decisions: <bounded list>
- Unresolved blockers: <bounded list or none>

## Step evidence

| Step | Status | Capability | Worker | Artifacts | Checks | Next gate |
| --- | --- | --- | --- | --- | --- | --- |
| spec | <status> | <identity> | <identity-or-current-context> | <locations> | <evidence> | plan |
| plan | <status> | <identity> | current-context | <locations> | <evidence> | implement |
| implement | <status> | <identity> | <executor> | <paths> | <commands-and-exit-codes> | review |
| review | <status> | <identity> | <independent-checker> | <report> | <coverage-and-scores> | iterate/ship |
| ship | <status-or-not-requested> | <identities> | <identity-or-current-context> | <commit-and-request> | <local-and-remote-proof> | done |

## Review anchor

- HEAD: <identifier>
- Plan identity: <identifier>
- Reviewed paths and fingerprints: <bounded manifest>
- Staged state: <bounded manifest>
- Unstaged state: <bounded manifest>
- Relevant untracked state: <bounded manifest>
- Validation evidence: <commands, targets, exit codes>
- Verdict: <ship-or-iterate>
- Completion score: <0-100>
- Quality score: <0-100>

## Iterations

| Wave | Hypothesis or fix list | Observable change | Verdict | Resume condition |
| --- | --- | --- | --- | --- |
| 1 | <value> | <evidence> | <value> | <value> |

## Delivery evidence

- Authorized effects: <exact original request>
- Commit identifiers: <verified values or not requested>
- Published branch: <verified remote state or not requested>
- Change request: <verified URL or not requested>
