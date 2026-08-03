# 02 - Gather

You collect only the evidence and decisions required for an actionable issue.

## Input

- Accept the issue boundary, template, required fields, user description, metadata, and available project or official evidence.

## Output

- Return objective or problem, actual and expected behavior, reproduction or use case, impact, proposed solution status, technical constraints, references, attachments, and QA or validation.

## Process

1. **Separate evidence.** Distinguish directly observed behavior, user report, logs, documentation, hypotheses, and selected decisions.
2. **Gather bug evidence.** Capture minimal reproduction, expected and actual behavior, frequency, environment category, impact, and sanitized evidence.
3. **Gather feature evidence.** Capture affected user, current limitation, desired outcome, boundaries, and observable completion criteria.
4. **Gather task evidence.** Capture deliverable, owner boundary, dependencies, exclusions, and verification.
5. **Gather documentation evidence.** Capture audience, current source, misleading or missing content, governing truth, and expected correction.
6. **Handle proposed solution.** Include a proposed solution only when the user, project, or verified prior decision supplies it. Otherwise mark it open or not applicable without inventing implementation.
7. **Gather references and attachments.** Validate URLs and safe files. Describe attachments without exposing local-only paths or private data.
8. **Inspect context.** Use relevant code, docs, errors, changelogs, and official current sources only when they improve actionability.
9. **Ask minimally.** Ask at most three focused questions when missing required information cannot be recovered safely.

## Stop conditions

- Stop when no concrete problem or desired outcome can be identified.
- Never reproduce destructive production behavior or include real secrets, accounts, private data, or unsafe attachments.
- Do not invent a proposed solution, constraint, reference, or validation method.

## Test

- Confirm that each required source or project field is evidenced, unavailable, or not applicable.
- Confirm that a maintainer can understand the outcome, scope, and validation.
