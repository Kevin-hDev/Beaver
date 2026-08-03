# 02 - Fetch

You retrieve the resolved ticket and only the related records required by the request.

## Input

- Accept one resolved tracker, project, identifier, and field boundary.

## Output

- Return current core fields and requested optional context in bounded pages.

## Process

1. **Query primary record.** You fetch one ticket through the read-only connector operation.
2. **Validate response.** You confirm identifier, project, field types, response size, and returned URL before use.
3. **Select core fields.** You keep title, description, type, status, priority, assignee, reporter when useful, labels, dates, and URL when available.
4. **Fetch optional context.** You fetch relationships in pages of at most 20 and comments in pages of at most 100 only when requested and supported. You keep a provider cursor, release each page after processing, and continue until the requested boundary or provider result is exhausted.
5. **Sanitize.** You remove secret-like values, hidden connector metadata, private email addresses when unnecessary, and unsafe embedded content.
6. **Record gaps.** You preserve the distinction between absent, redacted, unauthorized, unsupported, and failed fields.

## Stop conditions

- You stop on identifier mismatch, oversized or malformed response, access denial, or connector failure.
- You do not fall back to a broad web search for private or unidentified tracker content.
- You never invoke a mutation operation to obtain more detail.

## Test

- The returned record matches the resolved identifier, project, and tracker URL.
- Every optional page respects its bound, requested continuation is accounted for, and no mutation was invoked.
