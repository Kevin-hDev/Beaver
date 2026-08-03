# 03 - Audit Candidates

Audit every candidate independently and in parallel before allowing selection.

## Input

- Use the approved needs checklist, candidate table, and current evidence ledger.
- Use the verdict definitions and review dimensions in [candidate-audit-rubric.md](../references/candidate-audit-rubric.md).

## Output

- Return one independent pass, warning, or broken verdict and exactly three evidence-backed rationale bullets per candidate, plus the next workflow step.

## Process

1. **Prepare isolated briefs.** Give each reviewer only the complete needs, one candidate, its claims and sources, and the audit rubric. Do not reveal a preferred winner or another candidate's review.
2. **Launch in parallel.** Run one isolated reviewer per candidate in the same parallel wave. Keep the wave bounded to the two or three current candidates.
3. **Verify independently.** Require each reviewer to check component compatibility, maturity and support, recent known blockers, project constraints, security, integration, performance, deployment or distribution, team fit, and cost realism. Require current official evidence for unstable claims and recent primary issue or release evidence for material gotchas. Inspect at most 12 evidence items per reviewer batch, keep a source cursor, and continue until every audit dimension is resolved or explicitly blocked.
4. **Normalize output.** Assign pass, warning, or broken. Require exactly three bullets: compatibility and maturity; constraints and known risks; cost realism and evidence quality.
5. **Validate reviews.** Reject a review that lacks direct evidence, omits a checklist conflict, repeats another review without independent support, or gives a risk-free rationale. Request correction in batches of one attempt per reviewer, then start another bounded correction batch when useful evidence remains obtainable.
6. **Aggregate without ranking.** Add verdicts and three-bullet rationales to the candidate table. Do not choose a winner.
7. **Loop on total failure.** When every candidate is broken, show every verdict and the common blocker. Return to candidate generation when the candidates caused it, or to needs when requirements conflict. Continue through new bounded rounds until at least one candidate is viable, the user changes scope, or a true blocker ends the workflow.
8. **Advance conditionally.** Continue to selection only when at least one candidate passes or carries a mitigable warning.

## Stop conditions

- Stop when an isolated parallel review mechanism is unavailable. Do not replace independent reviews with one self-review or claim independence.
- Stop a review when required sources are inaccessible, and mark the candidate unready rather than guessing.
- Never advance an all-broken set or alter the needs to make a preferred candidate pass.

## Test

- Confirm that every candidate has an independently produced verdict and exactly three evidence-backed bullets.
- Confirm that reviewers did not receive a preferred answer or one another's conclusions.
- Confirm that an all-broken result returns to needs or candidates and never reaches selection.
