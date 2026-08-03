# 01 - Gather Needs

Collect and confirm every decision-driving need before proposing technology.

## Input

- Accept a new-project idea, product brief, or request to choose a stack or architecture.
- Accept optional existing project instructions and a preferred documentation destination.

## Output

- Return a complete 24-item checklist with 18 user-supplied decisions, six reserved derived choices, conflict notes, and explicit user confirmation.

## Process

1. **Validate scope.** Confirm that the request concerns a new project or undecided architecture. Separate design from any requested implementation.
2. **Load the checklist.** Copy [needs-checklist.md](../assets/needs-checklist.md) into conversation context. Show all four blocks before asking questions.
3. **Inspect conventions conditionally.** When a project already exists, inspect relevant instructions and documentation conventions in batches of at most 100 paths. Keep a continuation cursor and continue until the destination convention is resolved or explicitly unavailable.
4. **Ask by block.** Ask one user-input block per message and at most seven related questions in that message. Do not ask for the six derived choices.
5. **Make answers concrete.** Request numbers, units, examples, regions, platforms, deadlines, and ownership whenever words such as fast, secure, cheap, scalable, or simple would not distinguish candidates.
6. **Adapt without deleting.** Ask every item for every project family. Record not applicable only with a reason, such as no interactive client for a library or no tenant boundary for a local CLI.
7. **Check coherence.** Reconcile type, delivery surface, scale, integrations, security, performance, offline behavior, team capacity, budget, and deployment or distribution. Challenge contradictions and request a corrected or explicitly accepted constraint.
8. **Protect preferences.** Record preferred technologies as preferences, not decisions. Explain any visible conflict without softening it to please the user.
9. **Confirm.** Show the complete first three blocks, all not-applicable reasons, and conflict resolutions. Wait for explicit approval before proposing candidates.

## Stop conditions

- Stop when the request is implementation, migration of an existing stack, or an audit of an existing architecture rather than new-project design.
- Stop when a material item remains missing, contradictory, or too vague to compare candidates honestly.
- Do not fill derived choices, write a file, create a directory, or select technology in this action.

## Test

- Confirm that every one of the 18 user-input items contains a concrete answer or justified not-applicable value.
- Confirm that all six derived items remain undecided.
- Confirm that the user approved the complete needs record and that no project state changed.
