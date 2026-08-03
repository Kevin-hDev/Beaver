# Source Contract

Use one or more source specifications when evidence spans different origins.

## Specification

- `kind`: `conversation`, `file`, `diff`, or `review`.
- `label`: stable pointer shown to the user.
- `scope`: smallest slice that preserves evidence and context.
- `cursor`: next unread position or `complete`.
- `availability`: `ready`, `missing`, `unreadable`, `truncated`, or `unsafe`.

## Selection

- Use the current exchange for a conversation source.
- Use an explicit canonical project path and relevant section for a file source.
- Use an explicit working-tree change, revision, branch range, or supplied patch for a diff source.
- Use an explicit review artifact or supplied review text for a review source.
- Ask when multiple plausible sources would produce different learning.
- Select multiple specifications only when their combined evidence is necessary.

## Slice limits

- Read at most 100,000 inline characters per conversation slice.
- Read at most 256 KiB per file slice.
- Read at most 500 KiB or 50 changed files per diff or review slice.
- Continue ordered slices until the selected source is complete. Never treat a slice limit as a source limit.
