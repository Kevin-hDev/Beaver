# Async development integration contract

- Adapter identity: `__ADAPTER_ID__`
- Adapter version: `__ADAPTER_VERSION__`
- Verified format source: `__FORMAT_SOURCE__`
- Run entry: `action=run`
- Review entry: `action=review`
- Items per cycle: `1`
- Concurrency key: `__ITEM_ID_KEY__`
- Result artifact: `__RESULT_ARTIFACT__`
- Finalizer: `__FINALIZER_OPERATION__`
- Disable operation: `__DISABLE_OPERATION__`
- Validation evidence: `__VALIDATION_EVIDENCE__`

## Routing contract

- Change-request event: `action=review`
- Ticket run intent without active linked change: `action=run`
- Ticket run intent with active linked change: `no-op duplicate`
- Ticket review intent with one active linked change: `action=review`
- Ticket review intent without active linked change: `no-op not-reviewable`
- Ambiguous intent or relationship: `blocked or no-op; never guessed`

## Finalization contract

- You observe the external item, change request, result artifact, and critical operation results independently.
- You clear the working state and apply awaiting-review only after observed success or recovery, or blocked after observed failure.
- You deduplicate state transitions, audit records, and completion markers with stable run and completion keys.
- You propagate every critical finalization failure.

## Authorized effects

__AUTHORIZED_EFFECTS__

## Unsupported capabilities

__UNSUPPORTED_CAPABILITIES__
