# 02 - Collect Configuration

You collect a portable, bounded configuration and record exact authority for every possible effect.

## Input

- Use the detection report from action 01.
- Accept the user's execution preference, lifecycle names, triggers, limits, credential references, scheduling choice, and requested external effects.

## Output

- Return a validated configuration object that follows the configuration template.
- Return a separate effect contract with each effect, exact target, authority source, and `allowed` or `denied` state.
- Return unresolved choices without persisting configuration or performing an effect.

## Process

1. You read the configuration contract and present only execution paths supported by detected adapters: `local`, `remote`, or `both` when both are real.
2. You collect unique bounded names for ready, review, working, awaiting-review, and blocked states. You collect supported color and description metadata for each state when the tracker exposes them. You collect exact run and review triggers supported by the adapter.
3. You set one item per run cycle. You collect positive bounds for page size, maximum pages, review iterations, and any polling cadence.
4. You collect adapter identities from the detection report. You reject an identifier whose capabilities were not verified.
5. You collect credential bindings as opaque secure-store reference names. You may map an assignee or triggering actor to a reference and define one fallback reference, but you never ask for or inspect its value.
6. You keep optional package or capability installation separate from runtime credentials. You record the package source and exact package identities only when their real installation format is available.
7. You ask separately about every potential effect listed in the authority contract. You record commits, pushes, change-request creation or editing, comments, reactions, state changes, scheduling, smoke testing, cleanup, and external audit publication independently.
8. You default every unanswered effect to `denied`. You never infer publication or account changes from a request to configure files.
9. You validate the complete object for bounded values, distinct states, supported execution paths, safe strings, and absence of secret-looking values.
10. You show the configuration and effect contract with credential references redacted to identifiers only, then request confirmation before setup continues.

## Stop conditions

- You stop when a required adapter, lifecycle state, or bound remains unresolved.
- You stop when the user provides a secret value; you do not repeat it, store it, or include it in the result, and you direct the user to the secure store outside the conversation.
- You stop when requested effects exceed the available adapter capabilities or the user's exact authority.

## Test

- You confirm `items_per_cycle` equals `1` and every other limit is a positive bounded integer.
- You confirm all lifecycle state names are distinct and supported by the tracker adapter.
- You confirm every external effect has an explicit state and exact target.
- You scan the serialized object for credential values and confirm it contains only reference identifiers.
