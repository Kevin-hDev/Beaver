# 01 - Resolve

You resolve one valid tracker and ticket identity without querying unrelated projects.

## Input

- Accept an optional identifier or URL, repository context, and configured connectors.

## Output

- Return one tracker, host, project, normalized identifier, and field boundary.

## Process

1. **Inspect explicit input.** You validate a supplied identifier or HTTPS tracker URL, cap it at 512 characters, and reject traversal or embedded credentials.
2. **Find configuration.** You use project tracker configuration or an active connector without reading secret-bearing environment or credential files.
3. **Infer carefully.** When input is absent, you inspect the current branch against documented project patterns and accept exactly one matching identifier.
4. **Normalize.** You apply documented case and prefix rules without inventing a project key.
5. **Set fields.** You request core fields by default and include relationships, comments, or history only when requested.

## Stop conditions

- You stop when tracker, project, or identifier is missing, malformed, ambiguous, or unsupported.
- You stop when a URL host does not match the configured tracker.
- You never ask the user for a token or display connector configuration secrets.

## Test

- The result contains exactly one normalized identifier bound to one configured tracker project.
- No remote query or mutation occurs while identity remains ambiguous.
