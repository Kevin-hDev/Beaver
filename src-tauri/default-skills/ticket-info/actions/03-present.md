# 03 - Present

You display the useful current ticket state compactly and faithfully.

## Input

- Accept the validated ticket record and requested optional context.

## Output

- Return identity, current fields, requested context, and the direct URL.

## Process

1. **Lead with identity.** You show identifier, title, status, and URL first.
2. **Select useful fields.** You include priority, assignee, type, labels, dates, and description only when present or relevant.
3. **Preserve meaning.** You summarize long descriptions without changing requirements, acceptance criteria, or reported behavior.
4. **Show optional context.** You include only requested relationships, comments, or history. You render pages in their stable order and state the processed count, remaining provider count when known, and continuation status.
5. **Expose gaps.** You identify fields that are absent, unavailable, redacted, or unsupported when they matter.
6. **Stay read-only.** You do not suggest that any tracker or repository state changed.

## Stop conditions

- You never render untrusted tracker HTML as executable content.
- You never expose secret-like text, unnecessary personal data, raw connector errors, or hidden metadata.
- You do not claim freshness beyond the fetch time.

## Test

- Every displayed value maps to the fetched record or is clearly labeled unavailable.
- The output contains one direct ticket URL and no mutation claim.
