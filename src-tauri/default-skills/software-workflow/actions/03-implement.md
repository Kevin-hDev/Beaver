# 03 - Implement

You delegate the validated plan to an executor, observe the resulting artifacts and checks, and return only verified implementation state.

## Input

- Accept the complete plan and its recorded identity from `02`.
- Accept the original delivery contract and current repository evidence from the workflow ledger.
- Accept the independent review findings as an ordered fix list when `04` returned `iterate`.

## Output

- Return `status: implemented` with every phase complete and required validation green, or `status: blocked` with direct evidence and a resume condition.
- Return changed paths, observed validation commands and exit codes, repository state, phase results, and preserved unrelated changes.

## Process

1. **Gate on the plan.** You read the complete plan, verify its identity and `pending` or iteration-ready state, and reject any attempt to replace it with a summary. You record `status: in-progress` for an initial run. On an `iterate` return, you preserve the approved plan and attach findings as a fix list instead of editing its requirements.
2. **Discover the capability.** You select an available capability whose description says it implements an approved plan in verified phases. You inspect its described Git and artifact contract before you delegate.
3. **Brief an executor.** You spawn an implementation executor with the complete plan, applicable project rules, existing-change baseline, validation gates, original delivery contract, and optional fix list. You direct it to run the selected implementation capability completely. You prohibit implicit branch creation or switching and every external effect outside the original delivery contract.
4. **Run continuable waves.** You delegate a bounded executable slice at a time, preserve phase order and evidence, then continue later waves until all phases finish. You require a recorded hypothesis and expected signal for each repair attempt. You require a materially changed hypothesis after a failed attempt.
5. **Detect no progress.** You compare changed artifacts, failing checks, and diagnostic evidence after every attempt. You stop `blocked: no_progress` when two consecutive materially different hypotheses produce neither new evidence nor measurable improvement. You do not create a total-wave ceiling when progress remains observable.
6. **Observe the return.** You independently inspect the changed paths, relevant file contents, plan and phase statuses, repository status and diff, validation commands, exit codes, and preservation of unrelated work. You do not use the executor's report as proof.
7. **Resolve status.** You return `implemented` only when every phase is complete and every required validation exits successfully on the current state. You return `blocked` when a human-only decision, missing authority, unsafe overlap, unavailable dependency, validation failure without a supported next hypothesis, or no-progress condition remains.
8. **Record the transition.** You record the observed evidence in the ledger. You set workflow `status: implemented` only on success; otherwise you set `status: blocked` and stop the full workflow.

## Stop conditions

- You stop when the plan is missing, altered, unvalidated, or not implementation-ready.
- You stop when no matching implementation capability or isolated executor is available.
- You stop before any branch, commit, publication, deployment, or account effect not contained in the original request.
- You stop on unsafe overlap with unrelated changes, missing required access, a human-only decision, a failed required check without an evidence-backed next hypothesis, or demonstrated no progress.
- You stop the full workflow whenever observed status is `blocked`.

## Test

- Every planned phase is observably complete and every required validation command returns exit code 0 on the current state before status becomes `implemented`.
- The observed repository diff matches the plan and preserves unrelated user changes.
- An iteration fixes the current diff against the review findings without rewriting the approved plan.
- The ledger records direct artifact, validation, and repository evidence rather than trusting the executor's completion claim.
