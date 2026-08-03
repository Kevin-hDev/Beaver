# Documentation Validation Rubric

## Content

- Confirm that the document satisfies every contracted reader goal.
- Confirm that current, planned, deprecated, optional, and platform-specific behavior are distinguishable.
- Confirm that terminology, defaults, paths, commands, interfaces, errors, and limits match direct evidence.
- Confirm that prerequisites, safety boundaries, verification, and recovery appear where the reader can act on them.

## Structure and navigation

- Confirm that headings are unique and stable enough for anchors.
- Confirm that local links, anchors, images, included files, and navigation entries resolve.
- Confirm that the document has one clear owner for each subject and does not duplicate canonical content.
- Confirm that tables, code blocks, diagrams, lists, and callouts render legibly.
- Confirm that images and diagrams have useful text alternatives, heading levels remain ordered, and link labels describe their destination without relying on surrounding text.

## Procedures and examples

- Confirm that commands use an approved executable, separately validated arguments, a clean secret-free environment, disposable state, bounded time and output, and no network unless explicitly required and restricted.
- Confirm that examples use supported interfaces and their output shape matches current behavior.
- Confirm that migrations and state-changing procedures include backup, validation, rollback, and failure boundaries.
- Confirm that no secret, personal data, private endpoint, internal error detail, or machine-specific absolute path appears.

## Verdict

- Return `complete` when all required checks pass.
- Return `partial` only when the documentation is correct but named optional checks remain unavailable.
- Return `blocked` when required evidence, source consistency, links, examples, rendering, or documentation builds fail.
