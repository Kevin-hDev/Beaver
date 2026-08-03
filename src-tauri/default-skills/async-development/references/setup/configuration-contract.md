# Configuration contract

You keep the configuration secret-free and provider-independent. You validate it before any generated artifact or external effect.

## Required sections

- You record `version`, `execution`, `adapters`, `states`, `triggers`, `limits`, `schedule`, `credentials`, `effects`, and `audit`.
- You keep state and trigger names configurable while requiring unique, non-empty, bounded values. You keep state color and description metadata configurable when the tracker supports them.
- You require a positive review-iteration limit, page size, maximum pages per collection, polling interval when scheduled, and one item per run cycle.
- You store only opaque credential reference names and secure-store identifiers. You never store or request credential values.
- You identify each adapter by a verified capability record, not a guessed provider.
- You record each external effect separately and default it to `denied`.

## Validation

You reject unknown fields when the project schema is strict. You reject path traversal, duplicate state names, invalid trigger patterns, an unbounded collection, zero or negative limits, and an execution path without a usable adapter. You preserve existing user fields only when the schema permits them.

You update configuration with a temporary sibling file, an integrity check, and an atomic rename. You keep the last known valid file unchanged on any failure.
