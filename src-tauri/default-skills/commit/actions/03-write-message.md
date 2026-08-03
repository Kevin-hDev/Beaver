# 03 - Write Message

You derive and, when interactive, confirm one precise message from the final staged diff.

## Input

- Accept the verified staged diff, project convention, optional supplied message, and workflow choice.

## Output

- Return one final subject, optional body, optional verified footer, and approval state.

## Process

1. **Choose the convention.** Use documented repository practice or recent consistent history before the bundled fallback.
2. **Honor imposed text.** Use an explicitly imposed message exactly as supplied when it contains no secret or unsafe control content. Surface any project-convention or accuracy mismatch before recording; in `interactive`, wait for confirmation, and in `auto`, stop rather than silently rewrite it.
3. **Write the subject.** Use imperative form, keep it concise, and avoid vague words.
4. **Add a body selectively.** Explain why, risk, or migration context only when the subject cannot carry necessary durable context.
5. **Add references.** Include issue or breaking-change footers only when their identifiers and relationships are verified.
6. **Check accuracy.** Remove unrelated outcomes, invented validation, authors, co-authors, and unsupported claims.
7. **Confirm conditionally.** In `interactive`, show the complete message and wait for approval. In `auto`, continue only when the message is unambiguous.

## Stop conditions

- Stop when the staged diff changed after verification.
- Do not include secrets, internal errors, unnecessary local paths, or invented ticket identifiers.
- Stop when an imposed message is unsafe or the user rejects both the exact override and a proposed correction.

## Test

- Confirm that the message follows project convention and describes only the staged concern.
- Confirm that every body or footer claim is supported and interactive approval exists when required.
