# Action contract

You define a run's authority as an explicit intersection, never as whatever is technically possible.

## Required fields

| Field | You record |
| --- | --- |
| Objective | You record one observable end state. |
| Scope | You record exact project roots, paths, systems, data, and environments. |
| Allowed operations | You record read, create, edit, execute, install, or external-write classes separately. |
| Forbidden effects | You record destructive, financial, credential, publication, and unrelated effects. |
| Success predicate | You record a reproducible command or deterministic observation and its expected result. |
| Attempt boundary | You record batch size, maximum total attempts, and attempts already spent. |
| Time boundary | You record a deadline or maximum wall time. |
| Resource boundary | You record bounded output, process, service, quota, and monetary limits. |
| No-progress rule | You record progress signals and a finite consecutive-failure threshold. |
| Preservation plan | You record pre-existing work, expected touched files, and recovery steps. |

## Authority rules

- You treat the original request as authority only for effects necessary to deliver that request.
- You treat a requested deployment as authority for the named deployment target, not for account creation, billing changes, unrelated environments, or a later release.
- You treat a requested code fix as authority for project edits and relevant local checks, not for commits, pushes, tickets, releases, or deployments.
- You admit a destructive, financial, credential, account, or external-write effect only when the original request explicitly names that exact effect and target. You record a finite occurrence limit, safeguards, a cost ceiling when relevant, recovery, and verification.
- You require new user direction when an attempt needs a new scope, operation class, external target, financial exposure, destructive choice, secret, or larger total.
- You append contract amendments with their source and timestamp. You do not rewrite the original contract.

## Safe continuation

You permit another attempt only when all statements are true:

1. You have a distinct evidence-based hypothesis.
2. You have attempts, time, and resources remaining in the overall total.
3. You have not reached the no-progress threshold.
4. You can preserve current user work.
5. You can complete the attempt entirely inside the recorded authority.
