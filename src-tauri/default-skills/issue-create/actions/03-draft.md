# 03 - Draft

You write one complete issue and obtain explicit approval of its exact external content.

## Input

- Accept gathered evidence, type, project template, duplicate comparison, provider constraints, and requested metadata.

## Output

- Return an approved title, body, tracker project, type, labels, milestone, assignee, priority, references, and attachments.

## Process

1. **Write title.** Identify component and observable problem or desired outcome without vague urgency.
2. **Follow template.** Preserve required sections and mark irrelevant optional sections not applicable.
3. **Write source fields.** Include objective or problem, actual and expected behavior, reproduction or use case, impact, proposed solution when supported, technical constraints, references or attachments, and QA or validation.
4. **Link related work.** Reference non-duplicate related issues and explain scope differences.
5. **Apply metadata.** Validate labels, type, project, milestone, assignee, priority, references, and attachments against tracker options.
6. **Sanitize.** Remove secrets, personal data, raw internal paths, large logs, unsafe files, and unsupported claims.
7. **Validate with the user.** Show the exact destination, title, body, and every metadata value. Wait for explicit approval. Show material corrections again before creation.

## Stop conditions

- Stop when a required field remains unknown and affects actionability or destination.
- Do not prescribe a solution unless the request, template, or verified decision supports it.
- Do not broaden the issue or create while approval is absent, stale, conditional, or rejected.

## Test

- Confirm that the approved draft follows the project template and contains one actionable non-duplicate issue.
- Confirm that every metadata value exists and is explicit or mandatory.
