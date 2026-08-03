# Adapter and capability contract

You treat every external system as a separate adapter:

- The tracker adapter reads tickets, dependencies, lifecycle states, and discussions.
- The version-control adapter observes branches and commits.
- The change-request adapter reads or changes proposed changes and review threads.
- The integration adapter defines event payloads, jobs, result artifacts, and concurrency.
- The scheduler adapter creates, inspects, disables, and identifies schedules.
- The development-workflow adapter performs the complete plan, implementation, test, review, commit, and change-request lifecycle when its advertised contract includes those capabilities.

## Evidence you require

You accept an adapter only when you can inspect its installed tool schema, official current documentation, project-owned configuration, or a working executable interface. You record:

- its identity and version when available;
- the operations required by the selected sub-flow;
- the exact input and output fields used;
- the authentication reference name without its secret value;
- the idempotency or conditional-write mechanism;
- the verification query for every mutation;
- any missing capability.

You never infer compatibility from a provider name alone. You never generate an integration, command, query, or payload for an unavailable or undocumented interface. You return `unsupported` with the missing capability and the smallest safe next step.

## Mutation rules

You pass command arguments separately and never interpolate untrusted ticket text into a shell command. You validate identifiers, paths, state names, cursor sizes, and returned types. You bound every page and record the continuation cursor. You use conditional requests, revisions, compare-and-set operations, or another documented atomic primitive for locks. A sequence of independent label edits is not an atomic lock.
