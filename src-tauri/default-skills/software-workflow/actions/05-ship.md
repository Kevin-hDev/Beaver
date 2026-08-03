# 05 - Ship

You record and publish only the delivery explicitly requested after proving that the independent review still covers the current source state.

## Input

- Accept the `ship` verdict, review anchor, complete plan, phase results, and workflow ledger from `04`.
- Accept the original request's explicit delivery contract for commit, branch publication, and change-request creation.
- Use the current local and remote repository state.

## Output

- Return every created commit identifier and published branch state that the original delivery contract authorizes.
- Return the change-request URL when the original request explicitly asks for one.
- Return verified final local and remote state without claiming an unobserved effect.

## Process

1. **Gate authorization.** You read the original request, not a worker summary, and identify the exact requested delivery. You stop after the verified review handoff when it does not explicitly include delivery. You never infer commit, publication, or change-request authority from implementation scope alone.
2. **Gate the verdict.** You require a `ship` verdict whose plan identity and review anchor match the workflow ledger. You reject `iterate`, missing scores, incomplete acceptance coverage, and any superseded verdict.
3. **Gate the branch.** You resolve the default branch from validated repository metadata and remote state rather than assuming its name. You require the current branch to be an existing non-default branch. You stop with `contract_violation: on_default_branch` and create nothing when it is the default branch. You never create or switch a branch here.
4. **Prove freshness.** You compare the current `HEAD`, staged state, unstaged diff, relevant untracked paths, and source fingerprint with the review anchor. When the reviewed state was committed, you require changes after the reviewed identifier to contain only the selected workflow-ledger artifact. When it included uncommitted work, you require the current source fingerprint to match exactly. You stop and return to `04` after any source change.
5. **Discover delivery capabilities.** You select available capabilities by description for atomic commit, authorized branch publication, and provider-appropriate change-request creation. You invoke only the capabilities needed for the original delivery contract and let each own its checks and output.
6. **Record the reviewed change.** You give the commit capability the plan objective, reviewed change boundary, project conventions, and existing staged state. You verify every resulting commit from local history and confirm its source tree matches the reviewed fingerprint. You never include unrelated changes.
7. **Publish when requested.** You publish only the current non-default branch when the original request includes publication or a change request. You verify the remote branch contains the delivered commit and never force an update unless the original request explicitly requires it and the selected capability proves it safe.
8. **Open when requested.** You give the change-request capability the verified branch, base, full reviewed change, plan location or embedded plan identity, phase results, risks, and actual checks. You verify the returned URL and remote request state through the provider.
9. **Return observed state.** You record commit identifiers, remote branch state, change-request URL, and final checks in the ledger. You report unavailable or failed effects honestly and stop closed.

## Stop conditions

- You stop when the original request does not explicitly authorize the requested delivery effect.
- You stop with no commit on the default branch, detached state, unresolved branch identity, unsafe existing staged content, or unrelated overlapping changes.
- You stop and return to `04` when the review anchor is stale or cannot be reproduced.
- You stop when a required delivery capability, authenticated provider mechanism, remote, or unambiguous base is unavailable.
- You stop when commit, publication, or change-request verification fails; you never report success from an attempted command alone.

## Test

- No source content differs from the independently reviewed fingerprint at delivery time.
- Every returned commit exists in local history, contains only the reviewed scope, and reproduces the reviewed source state.
- Every reported published branch contains the delivered commit according to observed remote state.
- Every returned change-request URL is non-empty, belongs to the verified repository provider, and references the plan location or embedded plan identity.
- On the default branch or without explicit delivery authority, the action creates no commit, publication, or change request.
