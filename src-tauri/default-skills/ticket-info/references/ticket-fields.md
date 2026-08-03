# Ticket Fields

Read this reference when selecting optional ticket context.

- Core identity: identifier, title, type, status, priority, assignee, updated time, and URL.
- Core intent: description, acceptance criteria, and reproducible behavior when present.
- Relationships: parent, children, blockers, duplicates, dependencies, and linked requests; fetch and render pages of at most 20 until the requested boundary or provider result is exhausted.
- Comments: fetch only when requested, newest or most relevant first, process pages of at most 100, and continue until the requested boundary or provider result is exhausted.
- History: fetch only when the user asks how the ticket changed and the connector supports it read-only.
- Attachments: list safe metadata only; do not download content unless separately requested and validated.
- Personal data: omit private contact details and identities not needed to understand ownership or discussion.
