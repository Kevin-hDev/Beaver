# 04 - Create

You create exactly one approved issue and verify the tracker record.

## Input

- Accept the tracker mechanism, approved destination, title, body, project, type, and validated metadata.

## Output

- Return identifier, URL, title, type, body, verified metadata, and precise partial outcomes.

## Process

1. **Refresh approval scope.** Confirm that destination, draft, metadata, and provider state did not change after approval.
2. **Repeat duplicate check.** Run the narrow duplicate query immediately before creation.
3. **Create once.** Invoke the selected tracker mechanism with approved project, type, title, body, and metadata.
4. **Handle ambiguity.** Search for the issue before retrying an unclear provider response.
5. **Verify record.** Fetch the created issue and compare identifier, URL, title, body, type, labels, project, milestone, assignee, priority, references, and attachments.
6. **Report result.** Distinguish created, duplicate, blocked, provider-failed, and created-with-metadata-failure.

## Stop conditions

- Attempt creation at most twice and retry only a confirmed transient failure that created no issue.
- Never create a second issue to repair metadata on a successfully created one.
- Do not comment, transition, close, or add anything beyond approved or mandatory creation metadata.

## Test

- Confirm exactly one new issue with the approved identity and content.
- Confirm every reported metadata value or identify its partial failure.
