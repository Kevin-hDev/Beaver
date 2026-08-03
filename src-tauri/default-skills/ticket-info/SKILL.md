---
name: ticket-info
description: Retrieves one ticket from the configured tracker by ID, URL, or an unambiguous branch reference. Use to read ticket details and relationships. Not for creating, editing, commenting, assigning, transitioning, deleting, or implementing tickets.
---

# Ticket Info

You resolve one ticket safely, fetch it through the configured tracker, and present only useful current details without changing tracker state.

## Actions

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-resolve`](actions/01-resolve.md) | You receive a ticket lookup request | Tracker, project, normalized identifier, and query boundary |
| [`02-fetch`](actions/02-fetch.md) | One ticket identity is unambiguous | A bounded current ticket record and selected relationships |
| [`03-present`](actions/03-present.md) | The ticket record was fetched | A concise readable summary with URL and missing fields |

## Rules

- You remain read-only and never mutate the tracker, repository, branch, or local files.
- You prefer an explicit identifier or URL and infer from the current branch only when exactly one valid project pattern matches.
- You use the configured tracker connector and never search environment variables, credential files, or user data for tokens.
- You validate identifier length, format, host, project, and URL before querying.
- You fetch one primary ticket and process requested relationships in batches of at most 20 and comments in batches of at most 100 until the requested boundary or provider result is exhausted.
- You omit comments, attachments, history, custom fields, and relationships unless they help answer the request.
- You sanitize secret values, private personal data, raw internal errors, and hidden fields.
- You distinguish missing fields, unavailable fields, unsupported connector features, and access denial.
- You report only current tracker state actually returned by the connector.

## Resources

- Read [ticket-fields.md](references/ticket-fields.md) when selecting fields or presenting optional relationships and comments.
