# 03 - Draft

You write the complete request and obtain explicit user approval before external creation.

## Input

- Accept collected commits, behavior, risks, checks, head, base, mapped label, explicit metadata, requested state, and user overrides.

## Output

- Return an approved title, body, base, head, draft or ready state, mapped label, and explicit metadata.

## Process

1. **Choose template.** Use the repository's applicable request template and fall back only when none exists.
2. **Write title.** Summarize the complete change in repository style without copying an inaccurate single commit subject.
3. **Write summary.** Explain purpose and observable change before implementation detail.
4. **Write verification.** List only checks actually run or observed. Mark missing checks explicitly.
5. **Write risks.** Include breaking changes, migrations, rollback, security, screenshots, or follow-up sections only when relevant.
6. **Apply overrides.** Honor valid supplied title, body, base, state, and metadata without weakening required disclosures.
7. **Sanitize.** Remove credentials, private data, raw logs, local paths, and internal errors.
8. **Validate with the user.** Show full title, body, head, base, state, mapped label, and explicit metadata. Wait for explicit approval. Apply requested corrections and show the changed draft again.

## Stop conditions

- Stop when a required field cannot be answered honestly or marked not applicable.
- Do not invent issue links, screenshots, test results, reviewers, or deployment claims.
- Do not create while approval is missing, conditional, stale after a material draft change, or rejected.

## Test

- Confirm that title and body describe the complete range and satisfy the project template.
- Confirm that every check and risk is evidenced or unknown and the exact final draft is approved.
